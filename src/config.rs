use std::env;

pub fn debug_enabled() -> bool {
    env::var("POD_METRICS_EXPORTER_DEBUG").is_ok()
}
