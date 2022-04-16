mod runtime;

use once_cell::sync::Lazy;
use orderbook::telemetry::Tracer;

pub static TRACER: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        Tracer::new("orderbook", "debug").init(std::io::stdout);
    }
});

pub fn force_lazy() {
    Lazy::force(&TRACER);
}
