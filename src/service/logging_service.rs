use chrono::Local;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init_logging() {
    Builder::new()
        .format(|buf, record| {
            let symbol = match record.level() {
                Level::Info=>{
                    "ℹ️"
                }
                Level::Error => {
                    "❌"
                }
                Level::Warn => {
                    "⚠️"
                }
                Level::Debug => {
                    "🐛"
                }
                Level::Trace => {
                    "🔍"
                }
            };
            writeln!(
                buf,
                "{} {} - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                symbol,
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}
