use chrono::Local;
use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;

pub fn init_logging() {
    Builder::new()
        .format(|buf, record| {
            let symbol;
            match record.level() {
                Level::Info=>{
                    symbol = "ℹ️"
                }
                Level::Error => {
                    symbol = "❌"
                }
                Level::Warn => {
                    symbol = "⚠️"
                }
                Level::Debug => {
                    symbol = "🐛"
                }
                Level::Trace => {
                    symbol = "🔍"
                }
            }
            writeln!(
                buf,
                "{} {} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                symbol,
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}
