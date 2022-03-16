use chrono::Local;
use std::fmt::Write;
use tracing_subscriber;
use tracing_subscriber::fmt::time::FormatTime;

struct CustomTimeWrap;
impl FormatTime for CustomTimeWrap {
    fn format_time(&self, w: &mut dyn Write) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%H:%M:%S"))
    }
}
fn main() {
    let format = tracing_subscriber::fmt::format().with_timer(CustomTimeWrap);
    tracing_subscriber::fmt().event_format(format).init();
    tracing::info!("Hello World!!!")
}
