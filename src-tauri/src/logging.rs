use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Layer};
use ts_rs::TS;

const CAPACITY: usize = 2_000;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogLine {
    #[ts(type = "number")]
    pub timestamp: i64,

    pub level: String,

    pub target: String,
    pub message: String,
}

static BUFFER: OnceLock<Mutex<VecDeque<LogLine>>> = OnceLock::new();

fn buffer() -> &'static Mutex<VecDeque<LogLine>> {
    BUFFER.get_or_init(|| Mutex::new(VecDeque::with_capacity(CAPACITY)))
}

pub fn recent(limit: usize) -> Vec<LogLine> {
    let guard = buffer().lock().expect("log buffer lock poisoned");
    let skip = guard.len().saturating_sub(limit);
    guard.iter().skip(skip).cloned().collect()
}

pub fn clear() {
    buffer().lock().expect("log buffer lock poisoned").clear();
}

pub fn init() {
    let filter = EnvFilter::try_from_env("ZYNTAX_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info,zyntax=debug"));

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true).compact())
        .with(RingBufferLayer);

    match zyntax_store::Paths::from_env() {
        Ok(paths) => {
            let appender = tracing_appender::rolling::daily(paths.logs_dir(), "zyntax.log");
            registry
                .with(fmt::layer().with_ansi(false).with_writer(appender))
                .init();
        }
        Err(err) => {
            registry.init();
            tracing::warn!(%err, "could not resolve the log directory; logging to stdout only");
        }
    }
}

struct RingBufferLayer;

impl<S: Subscriber> Layer<S> for RingBufferLayer {
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let mut visitor = LineVisitor::default();
        event.record(&mut visitor);

        let metadata = event.metadata();
        let line = LogLine {
            timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
            level: metadata.level().to_string(),
            target: metadata.target().to_owned(),
            message: visitor.finish(),
        };

        let mut guard = buffer().lock().expect("log buffer lock poisoned");
        if guard.len() == CAPACITY {
            guard.pop_front();
        }
        guard.push_back(line);
    }
}

#[derive(Default)]
struct LineVisitor {
    message: String,
    fields: Vec<String>,
}

impl LineVisitor {
    fn finish(self) -> String {
        if self.fields.is_empty() {
            return self.message;
        }
        format!("{} ({})", self.message, self.fields.join(", "))
    }
}

impl Visit for LineVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}").trim_matches('"').to_owned();
        } else {
            self.fields.push(format!("{}={value:?}", field.name()));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_owned();
        } else {
            self.fields.push(format!("{}={value}", field.name()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_buffer_is_bounded() {
        {
            let mut guard = buffer().lock().unwrap();
            guard.clear();
            for index in 0..CAPACITY + 500 {
                if guard.len() == CAPACITY {
                    guard.pop_front();
                }
                guard.push_back(LogLine {
                    timestamp: index as i64,
                    level: "INFO".to_owned(),
                    target: "test".to_owned(),
                    message: format!("line {index}"),
                });
            }
        }

        let lines = recent(CAPACITY * 2);
        assert_eq!(lines.len(), CAPACITY, "must not grow without bound");

        assert_eq!(lines[0].timestamp, 500);

        clear();
    }

    #[test]
    fn recent_returns_at_most_the_requested_count() {
        clear();
        {
            let mut guard = buffer().lock().unwrap();
            for index in 0..10 {
                guard.push_back(LogLine {
                    timestamp: index,
                    level: "INFO".to_owned(),
                    target: "test".to_owned(),
                    message: String::new(),
                });
            }
        }

        let lines = recent(3);
        assert_eq!(lines.len(), 3);

        assert_eq!(lines[2].timestamp, 9);

        clear();
    }

    #[test]
    fn structured_fields_are_appended_to_the_message() {
        let visitor = LineVisitor {
            message: "hotkey registered".to_owned(),
            fields: vec!["accelerator=Ctrl+Alt+G".to_owned()],
        };

        assert_eq!(
            visitor.finish(),
            "hotkey registered (accelerator=Ctrl+Alt+G)"
        );
    }

    #[test]
    fn a_message_without_fields_is_left_alone() {
        let visitor = LineVisitor {
            message: "starting".to_owned(),
            fields: Vec::new(),
        };
        assert_eq!(visitor.finish(), "starting");
    }
}
