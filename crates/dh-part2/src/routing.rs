use dh_core::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thresholds {
    pub t_high: f32,
    pub t_low: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Authoritative,
    Review,
    Blank,
}

pub fn route(calibrated: Option<f32>, thresholds: Thresholds) -> Result<Route> {
    todo!("routing::route")
}
