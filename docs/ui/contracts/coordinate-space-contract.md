# Coordinate-Space Contract (T3/T4) — DRAFT

Source baseline: `9b497a3`. Files: `crates/dh-part3/src/view_transform.rs`, `crates/dh-part3/src/region.rs`, `crates/dh-app/src/viewer.rs`, `crates/dh-app/src/main.rs`, `crates/dh-part1/src/renderer.rs` (`effective_page_box`), `crates/dh-ui/ui/ViewerView.slint`, `crates/dh-ui/ui/RegionView.slint`.
Plan refs: v4 draft sections 5.6 (coordinate invariants), 7 (T3, T4), 9, 14 (pane resize risk), Appendix A item 6.
Plan section 5.6 requires the transform to be derived from code. The formulas below come from the code.

## Purpose
- Fix the canonical `PageTransform`, its units, the stored region form, and which side converts.
- Record the conversions as implemented, so Stream C can implement against them without changing the transform.

## Owner
- Rust only: `PageTransform` (`dh-part3/view_transform.rs`) is the single implementation.
- Rust callers: `dh-app/viewer.rs` (layout, fit zoom) and `dh-app/main.rs` (region accept, overlay projection, pan, pointer handling).
- Slint: display and pointer intent only. No page coordinates, no transform.
- Architecture owner approves any change to the canonical transform (plan 5.6, 16).

## Units
- PDF user space: points. Origin at lower-left, y up. Unrotated. Used for `page_box` and for the stored region.
- Logical px: Slint `length`. Rust receives it as f32 and widens to f64. Used by Slint and by `ViewerState` viewport and pan.
- Physical px: logical × `device_scale`. Used inside `PageTransform`.
- Zoom: physical px per PDF point, before `device_scale`. Logical px per point = `zoom`.

## PageTransform fields
| Field | Type | Unit | Meaning |
|---|---|---|---|
| `page_box` | BBox (dh_ir) | PDF points | Effective page box: CropBox if present, else MediaBox (`effective_page_box`). Bounds may be non-zero or negative. Width and height must be positive. |
| `rotation_degrees` | u16 | degrees | Normalized PDF `/Rotate`, clockwise. Only 0, 90, 180, 270. |
| `zoom` | f64 | px per point | Must be finite and > 0. Set from `ViewerState.zoom`. |
| `device_scale` | f64 | physical px per logical px | Must be finite and > 0. Set from `window.scale_factor()` at render (`start_render`). |
| `origin_x_px` | f64 | physical px, viewport-local | Viewport position of the top-left of the displayed (rotated) page. Includes centring and pan. Set to `image_x_logical × device_scale`. |
| `origin_y_px` | f64 | physical px, viewport-local | As above, for y. Set to `image_y_logical × device_scale`. |

Derived:
- `scale` = `zoom × device_scale` (physical px per PDF point).
- Display size px: width × scale, height × scale. Swapped for 90 and 270.

## Conversion (what the code does)
Let W = x1 − x0, H = y1 − y0 of `page_box`, s = scale.

Forward, PDF point (x, y) to physical viewport px:
1. u = x − x0; v = y1 − y.
2. Rotate (u, v) by `rotation_degrees`:
   - 0: (u, v)
   - 90: (H − v, u)
   - 180: (W − u, H − v)
   - 270: (v, W − u)
3. screen = (origin_x_px + rx × s, origin_y_px + ry × s).

Inverse, physical px to PDF point:
1. rx = (sx − origin_x_px) / s; ry = (sy − origin_y_px) / s.
2. Inverse rotate (rx, ry) using W and H:
   - 0: (rx, ry)
   - 90: (ry, H − rx)
   - 180: (W − rx, H − ry)
   - 270: (W − ry, rx)
3. x = x0 + u; y = y1 − v.

Rectangles: transform all four corners, then take min and max. Required for 90 and 270.

Region acceptance (`RegionSelection::to_page_user_space`):
- Screen rectangle → PDF rectangle (four corners).
- Reject if zero or negative area, non-finite, or any edge outside `page_box`. Rejected rectangles are not clipped.

## Stored form (authoritative)
- Stored: page-space `BBox` in unrotated PDF points, plus the 1-based page number. `dh-app` holds it as `RegionSelectionStore = Arc<Mutex<Option<BBox>>>`, with the page number in `ViewerState`.
- Not stored: screen px, logical px, zoom, pan, device scale, rotation-dependent values.
- The stored region does not change on zoom, pan, resize, collapse, snap, or locale change (plan 5.6).
- Stored values are not rounded. The transform does not round.

## Who converts (Rust)
- `current_page_transform` (`main.rs`): builds `PageTransform` from `ViewerState` and `viewer::layout`.
- `page_rect_from_screen_px` (`main.rs`): physical rectangle to `RegionSelection::to_page_user_space` to stored BBox. Called on every drag move and on release.
- `logical_rect_from_page_rect` (`main.rs`): stored BBox to `page_rect_to_screen` to ÷ `device_scale` to logical rect for the overlay.
- `candidate_screen_rect_px` (`main.rs`): drag arithmetic in physical px from the drag base.
- `refresh_region_overlay` (`main.rs`): calls the projection above; sets `region-x/y/width/height` and `region-visible`.
- `viewer::layout` and `fit_zoom` (`viewer.rs`): centring and display size, through `PageTransform` with origin 0.
- `refresh_pan` (`main.rs`): recomputes origin and re-projects the overlay from the stored BBox.

Slint does no page or physical conversion. The only arithmetic it does is adding a child's offset inside the overlay, to get viewport-local logical px (see Overlay rule).

## Overlay local-coordinate rule
- The overlay (`RegionOverlay`) sits at (0, 0) in the viewport `Rectangle`. Its local origin equals the viewport top-left.
- Pointer payloads are logical px relative to that origin. Child handles and the body add their own offset (`rect-x` plus handle x) so payloads are always overlay-local.
- The overlay draws only the rectangle Rust writes (`region-x/y/width/height`, logical px). It does not know the page, the page box, zoom, pan, rotation, or scale.
- The overlay never holds the `PageTransform`, and never holds the stored region.
- Transient drag state (`dragging`) lives in the overlay. Rust keeps the drag base in `RegionDrag` (physical px).

## Pointer events (overlay → MainWindow → dh-app)
| Event | Payload | When sent | Rust handling |
|---|---|---|---|
| `region-pointer-down(kind, x, y)` | kind: string; x, y: logical px, viewport-local | TouchArea down, when active | Multiply x, y by `device_scale`. Store `RegionDrag` with base rectangle. Draw clears the accepted region. |
| `region-pointer-move(x, y)` | logical px | Move while `dragging` | Candidate rectangle from base plus delta, then accept or reject. |
| `region-pointer-up(x, y)` | logical px | Up or cancel, while `dragging` | Same as move, then clears the drag. |
| `pan(dx, dy)` | logical px delta | ViewerPanel TouchArea, when not in selection mode | Adds to pan, re-lays out, re-projects overlay. |

Valid `kind` values: `draw`, `move`, `top-left`, `top`, `top-right`, `right`, `bottom-right`, `bottom`, `bottom-left`, `left`. Any other value falls back to the base rectangle.

## Rounding and scale
- No rounding in `PageTransform` or in the stored region.
- Render bitmap size only: `round()` in `positive_pixel_dimension` (`viewer.rs`). Display uses the float logical size.
- Device scale: read at render, not at each pointer event.

## Plan vs source conflicts
1. Pane resize recomputes the view transform and repaints (5.6, T3, risk 14): not wired. Viewport is read only in `start_render` and `refresh_pan`. No size-change callback exists.
2. Stored form (5.6): page space. Matches `dh-app` storage. The plan's change log says `region.rs` "stores screen pixels", which is out of date. The `RegionSelection` struct still holds screen px as a transient input.
3. Transform order, inverse, rotation, device scale (5.6 says to write them from code): done here. Rotation 0, 90, 180, 270 is supported.
4. Rounding (5.6 names it): none in the transform. Only the render bitmap size rounds.
5. DPI (5.6): device scale from `scale_factor()` at render only. Not refreshed on DPI change until the next render.
6. Overlay never converts (T4): the overlay adds child offsets. Local geometry, not page or physical conversion, but the rule is stricter in the plan.
7. Draw clears the accepted region on pointer down (not in plan): source behaviour.
8. Pointer cancel is treated as pointer up, so the candidate is accepted (not in plan).

## Stale or misleading doc comments
- `dh-part3/src/region.rs` module doc: "Overlays are re-projected from it on zoom, pan, rotation, and viewport (pane) resize." Re-projection runs on render, pan and region accept. Pane resize and a rotation action are not wired.
- `dh-part3/src/region.rs` `RegionSelection.rectangle`: doc says physical viewport px, which is correct for the field. The struct name and module doc suggest it is the stored form, which it is not. Field name is misleading.
- `dh-ui/ui/RegionView.slint` header: "It never converts coordinates." It adds child offsets. Accurate only if "convert" means page or physical conversion.
- `dh-app/src/main.rs` test `selection_survives_a_viewport_resize_with_page_coordinates_unchanged`: uses two synthetic states. It does not exercise the app's resize path, which does not exist.
- `dh-part3/src/view_transform.rs` `origin_*_px` doc: does not say the origin is the top-left of the displayed page, not of the unrotated page. Missing detail, not wrong.

## Open decisions
1. Make the page-space BBox (PDF points, unrotated, 1-based page) the only stored region form? yes/no
2. Store exact f64 page coordinates with no rounding? yes/no
3. Add a viewport-size-change path that re-runs layout and overlay projection (pane resize)? yes/no
4. Pointer cancel: commit the candidate (current), or revert to the last accepted rectangle? choose: commit / revert
5. Approve the source transform order (rotate, scale, origin) and inverse as canonical, backed by the existing tests? yes/no
6. Keep child-offset arithmetic in the overlay as local geometry? yes/no
7. Draw clears the accepted region at pointer down (current)? yes/no
8. Device scale: keep read-at-render, or re-read on each pointer event? choose: render / pointer
9. Rename `RegionSelection.rectangle` to `screen_rectangle` in a separate PR? yes/no
10. Fix the stale comments in a doc-only PR before T4 starts? yes/no

## Decisions (accepted)
- Page-space BBox (PDF points, unrotated, 1-based page) is the only stored region form. Exact f64, no rounding.
- Viewport-size change re-runs layout and overlay projection (pane resize).
- Pointer cancel keeps current behaviour: commit the candidate. Revert not adopted; change needs separate owner approval.
- Transform order (rotate, scale, origin) and inverse approved as canonical, backed by existing tests.
- Overlay keeps child-offset arithmetic as local geometry.
- Draw clears the accepted region at pointer down.
- Device scale read at render time.
- Rename `RegionSelection.rectangle` to `screen_rectangle` and fix stale comments in separate doc-only PRs before T4 starts.
