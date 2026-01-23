pub use tracing::*;

pub fn init_tracing_subscriber_log() {
    use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
    let span = FmtSpan::NEW | FmtSpan::CLOSE;
    let filter = EnvFilter::from_default_env();
    tracing_subscriber::fmt().with_span_events(span).with_env_filter(filter).init();
}
