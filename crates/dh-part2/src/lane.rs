#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneOptions {
    pub available_ram_bytes: u64,
    pub peak_rss_per_job_bytes: u64,
    pub default_max: usize,
    pub backend_max_concurrent: usize,
}

pub fn lane_size(options: &LaneOptions) -> usize {
    if options.peak_rss_per_job_bytes == 0 {
        return 1;
    }
    let usable = options.available_ram_bytes.saturating_mul(60) / 100;
    let computed = (usable / options.peak_rss_per_job_bytes) as usize;
    computed
        .clamp(1, options.default_max.max(1))
        .min(options.backend_max_concurrent.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_size_is_clamped_and_bounded() {
        let options = LaneOptions {
            available_ram_bytes: 16 * 1024 * 1024 * 1024,
            peak_rss_per_job_bytes: 1024 * 1024 * 1024,
            default_max: 4,
            backend_max_concurrent: 2,
        };
        assert_eq!(lane_size(&options), 2);
    }

    #[test]
    fn lane_size_never_drops_below_one() {
        let options = LaneOptions {
            available_ram_bytes: 0,
            peak_rss_per_job_bytes: 1,
            default_max: 4,
            backend_max_concurrent: 4,
        };
        assert_eq!(lane_size(&options), 1);
    }
}
