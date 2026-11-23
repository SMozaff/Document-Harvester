//! Stage 2 viewer state and geometry.
//!
//! Every conversion between the rendered viewport and PDF user space goes
//! through `dh_part3::PageTransform`. This module only decides how much of a
//! page fits the viewport and where the image is placed; it never duplicates
//! zoom, pan, rotation or page-box arithmetic.

use std::path::PathBuf;

use dh_core::{DhError, Result};
use dh_part1::renderer::PageInfo;
use dh_part3::PageTransform;

/// Viewer session state. The PDF path is always the private app-data copy
/// recorded by `dh_project::ProjectStore::import_pdf`; the original source path
/// is never stored here.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerState {
    pub project_id: String,
    pub pdf_path: Option<PathBuf>,
    pub page_number: u32,
    pub page_count: u32,
    pub zoom: f64,
    pub pan_x_logical: f64,
    pub pan_y_logical: f64,
    pub viewport_width_logical: f64,
    pub viewport_height_logical: f64,
    pub device_scale: f64,
    pub info: Option<PageInfo>,
    pub generation: u64,
}

impl ViewerState {
    pub fn new() -> Self {
        Self {
            project_id: String::new(),
            pdf_path: None,
            page_number: 1,
            page_count: 0,
            zoom: 1.0,
            pan_x_logical: 0.0,
            pan_y_logical: 0.0,
            viewport_width_logical: 0.0,
            viewport_height_logical: 0.0,
            device_scale: 1.0,
            info: None,
            generation: 0,
        }
    }

    pub fn move_page(&mut self, delta: i32) -> bool {
        let target = self.page_number as i64 + delta as i64;
        if target < 1 || target > self.page_count as i64 {
            return false;
        }
        self.page_number = target as u32;
        true
    }

    pub fn change_zoom(&mut self, factor: f64) {
        let zoom = self.zoom * factor;
        if zoom.is_finite() && zoom > 0.0 {
            self.zoom = zoom;
        }
    }

    pub fn pan(&mut self, dx_logical: f64, dy_logical: f64) {
        if dx_logical.is_finite() && dy_logical.is_finite() {
            self.pan_x_logical += dx_logical;
            self.pan_y_logical += dy_logical;
        }
    }
}

/// Where the page image is placed and the pixel size at which it is rendered.
#[derive(Debug, Clone, PartialEq)]
pub struct PageLayout {
    pub image_x_logical: f64,
    pub image_y_logical: f64,
    pub image_width_logical: f64,
    pub image_height_logical: f64,
    pub render_width_px: u32,
    pub render_height_px: u32,
}

impl PageLayout {
    /// The transform used for rendering and for any later region selection.
    /// Exercised by the viewer geometry tests; later overlay work calls it.
    #[cfg(test)]
    pub fn transform(&self, info: &PageInfo, zoom: f64, device_scale: f64) -> PageTransform {
        PageTransform {
            page_box: info.page_box,
            rotation_degrees: info.rotation_degrees,
            zoom,
            device_scale,
            origin_x_px: self.image_x_logical * device_scale,
            origin_y_px: self.image_y_logical * device_scale,
        }
    }
}

fn display_size(info: &PageInfo, zoom: f64, device_scale: f64) -> Result<(f64, f64)> {
    PageTransform {
        page_box: info.page_box,
        rotation_degrees: info.rotation_degrees,
        zoom,
        device_scale,
        origin_x_px: 0.0,
        origin_y_px: 0.0,
    }
    .display_size_px()
}

fn positive_pixel_dimension(value: f64) -> Result<u32> {
    if !value.is_finite() || value < 1.0 {
        return Err(DhError::InvalidInput(
            "the rendered page has no usable pixel area".to_owned(),
        ));
    }
    Ok(value.round() as u32)
}

/// Zoom that fits the rotated page inside a viewport of logical pixels.
pub fn fit_zoom(
    info: &PageInfo,
    viewport_width_logical: f64,
    viewport_height_logical: f64,
    device_scale: f64,
) -> Result<f64> {
    if viewport_width_logical <= 0.0
        || viewport_height_logical <= 0.0
        || !viewport_width_logical.is_finite()
        || !viewport_height_logical.is_finite()
    {
        return Err(DhError::InvalidInput(
            "the viewer viewport has no usable area".to_owned(),
        ));
    }
    let (page_width, page_height) = display_size(info, 1.0, device_scale)?;
    let zoom = (viewport_width_logical * device_scale / page_width)
        .min(viewport_height_logical * device_scale / page_height);
    if !zoom.is_finite() || zoom <= 0.0 {
        return Err(DhError::InvalidInput(
            "the page cannot be fitted to the viewport".to_owned(),
        ));
    }
    Ok(zoom)
}

/// Compute the image placement and render size through `PageTransform`.
pub fn layout(
    info: &PageInfo,
    zoom: f64,
    pan_logical: (f64, f64),
    viewport_logical: (f64, f64),
    device_scale: f64,
) -> Result<PageLayout> {
    if !pan_logical.0.is_finite() || !pan_logical.1.is_finite() {
        return Err(DhError::InvalidInput("pan offset is not finite".to_owned()));
    }
    if !device_scale.is_finite() || device_scale <= 0.0 {
        return Err(DhError::InvalidInput(
            "display scale must be positive and finite".to_owned(),
        ));
    }

    let transform = PageTransform {
        page_box: info.page_box,
        rotation_degrees: info.rotation_degrees,
        zoom,
        device_scale,
        origin_x_px: 0.0,
        origin_y_px: 0.0,
    };
    let (render_width, render_height) = transform.display_size_px()?;
    let image_width_logical = render_width / device_scale;
    let image_height_logical = render_height / device_scale;
    Ok(PageLayout {
        image_x_logical: (viewport_logical.0 - image_width_logical) / 2.0 + pan_logical.0,
        image_y_logical: (viewport_logical.1 - image_height_logical) / 2.0 + pan_logical.1,
        image_width_logical,
        image_height_logical,
        render_width_px: positive_pixel_dimension(render_width)?,
        render_height_px: positive_pixel_dimension(render_height)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dh_ir::BBox;
    use dh_part3::{Point, RegionSelection};

    fn info(box_bounds: BBox, rotation_degrees: u16) -> PageInfo {
        PageInfo {
            page_count: 2,
            page_number: 1,
            width_points: box_bounds.x1 - box_bounds.x0,
            height_points: box_bounds.y1 - box_bounds.y0,
            rotation_degrees,
            page_box: box_bounds,
        }
    }

    #[test]
    fn zoom_levels_drive_the_physical_render_size_through_page_transform() {
        let page = info(
            BBox {
                x0: 17.0,
                y0: 29.0,
                x1: 217.0,
                y1: 129.0,
            },
            0,
        );
        for (zoom, expected_width, expected_height) in [
            (1.0, 200.0, 100.0),
            (2.5, 500.0, 250.0),
            (4.0, 800.0, 400.0),
        ] {
            let layout = layout(&page, zoom, (0.0, 0.0), (900.0, 600.0), 1.0).unwrap();
            assert_eq!(
                (
                    layout.render_width_px as f64,
                    layout.render_height_px as f64
                ),
                (expected_width, expected_height)
            );
            assert_eq!(layout.image_width_logical, expected_width);
            assert_eq!(layout.image_height_logical, expected_height);
        }
    }

    #[test]
    fn quarter_turns_swap_the_rendered_page_dimensions() {
        let page = info(
            BBox {
                x0: 10.0,
                y0: 20.0,
                x1: 210.0,
                y1: 120.0,
            },
            90,
        );
        let quarter_turn = layout(&page, 1.0, (0.0, 0.0), (900.0, 600.0), 2.0).unwrap();
        assert_eq!(
            (quarter_turn.render_width_px, quarter_turn.render_height_px),
            (200, 400)
        );
        assert_eq!(
            (
                quarter_turn.image_width_logical,
                quarter_turn.image_height_logical
            ),
            (100.0, 200.0)
        );

        let page_180 = info(page.page_box, 180);
        let half_turn = layout(&page_180, 1.0, (0.0, 0.0), (900.0, 600.0), 2.0).unwrap();
        assert_eq!(
            (half_turn.render_width_px, half_turn.render_height_px),
            (400, 200)
        );
    }

    #[test]
    fn fit_uses_the_viewport_and_display_scale() {
        let page = info(
            BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 200.0,
                y1: 100.0,
            },
            0,
        );
        let zoom = fit_zoom(&page, 600.0, 400.0, 2.0).unwrap();
        assert_eq!(zoom, 3.0);
        let layout = layout(&page, zoom, (0.0, 0.0), (600.0, 400.0), 2.0).unwrap();
        assert_eq!(
            (layout.render_width_px, layout.render_height_px),
            (1200, 600)
        );
        assert_eq!(
            (layout.image_width_logical, layout.image_height_logical),
            (600.0, 300.0)
        );
    }

    #[test]
    fn pan_moves_the_image_without_resizing_it() {
        let page = info(
            BBox {
                x0: -10.0,
                y0: 5.0,
                x1: 90.0,
                y1: 55.0,
            },
            0,
        );
        let centered = layout(&page, 2.0, (0.0, 0.0), (800.0, 500.0), 1.0).unwrap();
        let panned = layout(&page, 2.0, (-35.0, 125.0), (800.0, 500.0), 1.0).unwrap();
        assert_eq!(centered.render_width_px, panned.render_width_px);
        assert_eq!(centered.render_height_px, panned.render_height_px);
        assert_eq!(panned.image_x_logical - centered.image_x_logical, -35.0);
        assert_eq!(panned.image_y_logical - centered.image_y_logical, 125.0);
    }

    #[test]
    fn the_viewer_transform_maps_selections_with_the_canonical_path() {
        let page = info(
            BBox {
                x0: 10.0,
                y0: 20.0,
                x1: 210.0,
                y1: 120.0,
            },
            90,
        );
        let layout = layout(&page, 2.5, (25.0, -15.0), (900.0, 600.0), 2.0).unwrap();
        let transform = layout.transform(&page, 2.5, 2.0);
        let top_left = transform
            .page_to_screen(Point { x: 10.0, y: 20.0 })
            .unwrap();
        assert_eq!(
            (top_left.x, top_left.y),
            (layout.image_x_logical * 2.0, layout.image_y_logical * 2.0)
        );

        let selection = RegionSelection {
            page: 1,
            rectangle: BBox {
                x0: top_left.x + 10.0,
                y0: top_left.y + 10.0,
                x1: top_left.x + 50.0,
                y1: top_left.y + 40.0,
            },
        };
        let page_rectangle = selection.to_page_user_space(&transform).unwrap();
        assert!(page_rectangle.x0 >= page.page_box.x0);
        assert!(page_rectangle.y1 <= page.page_box.y1);
    }

    #[test]
    fn degenerate_viewports_and_negative_zoom_are_rejected() {
        let page = info(
            BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 100.0,
            },
            0,
        );
        assert!(fit_zoom(&page, 0.0, 400.0, 1.0).is_err());
        assert!(layout(&page, -1.0, (0.0, 0.0), (800.0, 600.0), 1.0).is_err());
        assert!(layout(&page, 1.0, (f64::NAN, 0.0), (800.0, 600.0), 1.0).is_err());
    }
}
