use chrono::Local;
use std::fmt;
use tracing::{Event, Level, Subscriber};
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::{
    FmtContext, FormatEvent, FormatFields,
    format::{DefaultFields, Writer},
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

struct EmojiFormatter;

impl<S, N> FormatEvent<S, N> for EmojiFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let symbol = match *event.metadata().level() {
            Level::INFO => "ℹ️",
            Level::ERROR => "❌",
            Level::WARN => "⚠️",
            Level::DEBUG => "🐛",
            Level::TRACE => "🔍",
        };
        write!(
            writer,
            "{} {} - ",
            Local::now().format("%Y-%m-%dT%H:%M:%S"),
            symbol,
        )?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn init_logging() {
    let _ = LogTracer::init();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .event_format(EmojiFormatter)
        .fmt_fields(DefaultFields::new());

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .try_init();
}
