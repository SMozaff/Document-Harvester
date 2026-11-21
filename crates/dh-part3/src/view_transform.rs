//! Canonical transforms between PDF user space and the rendered page viewport.
//!
//! PDF coordinates use the page's unrotated user space (origin at the lower-left).
//! Viewport coordinates use a top-left origin and physical screen pixels. The
//! `origin_*_px` values include the viewport placement and current pan offset.
//! `device_scale` converts logical UI pixels to physical pixels; callers that
//! already provide physical coordinates should use 1.0.

use dh_core::{DhError, Result};
use dh_ir::BBox;

/// A point in either PDF user space or physical viewport pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// The transform for one rendered PDF page.
///
/// `page_box` must be the effective page box in PDF user space (normally the
/// CropBox, falling back to MediaBox when the PDF has no CropBox). Its bounds
/// may be non-zero or negative. Rotation is the normalized PDF page rotation:
/// clockwise, in quarter turns only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageTransform {
    pub page_box: BBox,
    pub rotation_degrees: u16,
    pub zoom: f64,
    pub device_scale: f64,
    pub origin_x_px: f64,
    pub origin_y_px: f64,
}

impl PageTransform {
    /// Convert a PDF user-space point to physical viewport pixels.
    pub fn page_to_screen(&self, point: Point) -> Result<Point> {
        self.validate()?;
        if !is_finite(point.x) || !is_finite(point.y) {
            return Err(invalid("page point must contain finite coordinates"));
        }

        let (width, height) = self.page_dimensions();
        let u = point.x - self.page_box.x0;
        let v = self.page_box.y1 - point.y;
        let (rotated_x, rotated_y) = self.rotate_forward(u, v, width, height);
        let scale = self.scale();
        Ok(Point {
            x: self.origin_x_px + rotated_x * scale,
            y: self.origin_y_px + rotated_y * scale,
        })
    }

    /// Convert a physical viewport point to PDF user space.
    pub fn screen_to_page(&self, point: Point) -> Result<Point> {
        self.validate()?;
        if !is_finite(point.x) || !is_finite(point.y) {
            return Err(invalid("screen point must contain finite coordinates"));
        }

        let (width, height) = self.page_dimensions();
        let scale = self.scale();
        let rotated_x = (point.x - self.origin_x_px) / scale;
        let rotated_y = (point.y - self.origin_y_px) / scale;
        let (u, v) = self.rotate_inverse(rotated_x, rotated_y, width, height);
        Ok(Point {
            x: self.page_box.x0 + u,
            y: self.page_box.y1 - v,
        })
    }

    /// Transform all four corners of a PDF rectangle into a screen-aligned box.
    ///
    /// Transforming all corners (rather than only two diagonal corners) is
    /// required for 90-degree and 270-degree page rotations.
    pub fn page_rect_to_screen(&self, rectangle: BBox) -> Result<BBox> {
        validate_box(rectangle, "PDF rectangle")?;
        let points = [
            Point { x: rectangle.x0, y: rectangle.y0 },
            Point { x: rectangle.x0, y: rectangle.y1 },
            Point { x: rectangle.x1, y: rectangle.y0 },
            Point { x: rectangle.x1, y: rectangle.y1 },
        ];
        self.points_to_box(points.map(|point| self.page_to_screen(point)))?
    }

    /// Transform all four corners of a screen rectangle into PDF user space.
    pub fn screen_rect_to_page(&self, rectangle: BBox) -> Result<BBox> {
        validate_box(rectangle, "screen rectangle")?;
        let points = [
            Point { x: rectangle.x0, y: rectangle.y0 },
            Point { x: rectangle.x0, y: rectangle.y1 },
            Point { x: rectangle.x1, y: rectangle.y0 },
            Point { x: rectangle.x1, y: rectangle.y1 },
        ];
        self.points_to_box(points.map(|point| self.screen_to_page(point)))?
    }

    /// Return the rendered page width and height in physical screen pixels.
    pub fn display_size_px(&self) -> Result<(f64, f64)> {
        self.validate()?;
        let (width, height) = self.page_dimensions();
        let scale = self.scale();
        if self.rotation_degrees == 90 || self.rotation_degrees == 270 {
            Ok((height * scale, width * scale))
        } else {
            Ok((width * scale, height * scale))
        }
    }

    fn validate(&self) -> Result<()> {
        validate_box(self.page_box, "page box")?;
        if !matches!(self.rotation_degrees, 0 | 90 | 180 | 270) {
            return Err(invalid("page rotation must be 0, 90, 180 or 270 degrees"));
        }
        if !is_finite(self.zoom) || self.zoom <= 0.0 {
            return Err(invalid("zoom must be finite and greater than zero"));
        }
        if !is_finite(self.device_scale) || self.device_scale <= 0.0 {
            return Err(invalid("device scale must be finite and greater than zero"));
        }
        if !is_finite(self.origin_x_px) || !is_finite(self.origin_y_px) {
            return Err(invalid("viewport origin must contain finite coordinates"));
        }
        Ok(())
    }

    fn page_dimensions(&self) -> (f64, f64) {
        (self.page_box.x1 - self.page_box.x0, self.page_box.y1 - self.page_box.y0)
    }

    fn scale(&self) -> f64 {
        self.zoom * self.device_scale
    }

    // Input coordinates here are top-left-relative coordinates in the
    // unrotated page. Positive PDF /Rotate values are clockwise.
    fn rotate_forward(&self, u: f64, v: f64, width: f64, height: f64) -> (f64, f64) {
        match self.rotation_degrees {
            0 => (u, v),
            90 => (height - v, u),
            180 => (width - u, height - v),
            270 => (v, width - u),
            _ => unreachable!("rotation is validated before use"),
        }
    }

    fn rotate_inverse(&self, x: f64, y: f64, width: f64, height: f64) -> (f64, f64) {
        match self.rotation_degrees {
            0 => (x, y),
            90 => (y, height - x),
            180 => (width - x, height - y),
            270 => (width - y, x),
            _ => unreachable!("rotation is validated before use"),
        }
    }

    fn points_to_box(
        &self,
        points: [Result<Point>; 4],
    ) -> Result<BBox> {
        let points = points.map(|point| point).into_iter().collect::<Result<Vec<_>>>()?;
        let mut x0 = f64::INFINITY;
        let mut y0 = f64::INFINITY;
        let mut x1 = f64::NEG_INFINITY;
        let mut y1 = f64::NEG_INFINITY;
        for point in points {
            x0 = x0.min(point.x);
            y0 = y0.min(point.y);
            x1 = x1.max(point.x);
            y1 = y1.max(point.y);
        }
        Ok(BBox { x0, y0, x1, y1 })
    }
}

fn validate_box(rectangle: BBox, label: &str) -> Result<()> {
    if !is_finite(rectangle.x0)
        || !is_finite(rectangle.y0)
        || !is_finite(rectangle.x1)
        || !is_finite(rectangle.y1)
    {
        return Err(invalid(&format!("{label} must contain finite coordinates")));
    }
    if rectangle.x1 <= rectangle.x0 || rectangle.y1 <= rectangle.y0 {
        return Err(invalid(&format!("{label} must have positive width and height")));
    }
    Ok(())
}

fn is_finite(value: f64) -> bool {
    value.is_finite()
}

fn invalid(message: &str) -> DhError {
    DhError::InvalidInput(message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transform(rotation_degrees: u16) -> PageTransform {
        PageTransform {
            // Deliberately non-zero and non-origin page bounds.
            page_box: BBox { x0: 17.0, y0: 29.0, x1: 217.0, y1: 129.0 },
            rotation_degrees,
            zoom: 1.5,
            device_scale: 2.0,
            origin_x_px: 80.0,
            origin_y_px: 45.0,
        }
    }

    fn close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "actual={actual}, expected={expected}");
    }

    #[test]
    fn zero_rotation_maps_page_box_to_viewport() {
        let t = transform(0);
        let top_left = t.page_to_screen(Point { x: 17.0, y: 129.0 }).unwrap();
        let bottom_right = t.page_to_screen(Point { x: 217.0, y: 29.0 }).unwrap();
        assert_eq!(top_left, Point { x: 80.0, y: 45.0 });
        assert_eq!(bottom_right, Point { x: 680.0, y: 345.0 });
        assert_eq!(t.display_size_px().unwrap(), (600.0, 300.0));
    }

    #[test]
    fn rotated_display_dimensions_are_swapped() {
        assert_eq!(transform(90).display_size_px().unwrap(), (300.0, 600.0));
        assert_eq!(transform(270).display_size_px().unwrap(), (300.0, 600.0));
        assert_eq!(transform(180).display_size_px().unwrap(), (600.0, 300.0));
    }

    #[test]
    fn page_and_screen_points_round_trip_for_every_quarter_turn() {
        let source = Point { x: 63.25, y: 91.75 };
        for rotation in [0, 90, 180, 270] {
            let t = transform(rotation);
            let screen = t.page_to_screen(source).unwrap();
            let restored = t.screen_to_page(screen).unwrap();
            close(restored.x, source.x);
            close(restored.y, source.y);
        }
    }

    #[test]
    fn rectangle_round_trip_uses_all_four_corners() {
        let source = BBox { x0: 31.0, y0: 42.0, x1: 113.0, y1: 107.0 };
        for rotation in [0, 90, 180, 270] {
            let t = transform(rotation);
            let screen = t.page_rect_to_screen(source).unwrap();
            let restored = t.screen_rect_to_page(screen).unwrap();
            close(restored.x0, source.x0);
            close(restored.y0, source.y0);
            close(restored.x1, source.x1);
            close(restored.y1, source.y1);
        }
    }

    #[test]
    fn zoom_device_scale_and_pan_are_applied_together() {
        let t = transform(0);
        let point = t.page_to_screen(Point { x: 27.0, y: 119.0 }).unwrap();
        // 10 PDF units at zoom 1.5 and device scale 2 = 30 physical pixels.
        assert_eq!(point, Point { x: 110.0, y: 75.0 });
    }

    #[test]
    fn invalid_geometry_is_rejected() {
        let mut t = transform(0);
        t.rotation_degrees = 45;
        assert!(t.page_to_screen(Point { x: 20.0, y: 40.0 }).is_err());

        t = transform(0);
        t.zoom = 0.0;
        assert!(t.display_size_px().is_err());

        t = transform(0);
        t.page_box.x1 = t.page_box.x0;
        assert!(t.display_size_px().is_err());

        t = transform(0);
        assert!(t.screen_to_page(Point { x: f64::NAN, y: 0.0 }).is_err());
    }
}
