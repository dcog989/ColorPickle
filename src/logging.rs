use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::ValueEnum;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

use crate::config::APP_NAME;
use crate::state;

const LOG_DIRECTORY: &str = "log";
const LOG_FILE: &str = "colorpickle.log";
const RETAINED_LOG_FILES: usize = 7;

/// Verbosity of the file and stderr logs, unless `RUST_LOG` overrides it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

/// Installs the global subscriber: a daily-rotating file log in the XDG state
/// directory, mirrored to stderr.
///
/// The returned guard must stay alive for the whole run; dropping it discards
/// lines still queued for the file. `None` means the file log could not be
/// opened, in which case stderr logging still works.
pub fn init(level: LogLevel) -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{APP_NAME}={}", level.as_str())));
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal());
    let subscriber = tracing_subscriber::registry().with(filter);

    let Some(directory) = directory() else {
        subscriber.with(stderr_layer).init();
        eprintln!("colorpickle: no state directory available; logging to stderr only");
        return None;
    };

    let file = appender(&directory).inspect_err(|error| {
        eprintln!("colorpickle: {error:#}; logging to stderr only");
    });
    let Ok((writer, guard)) = file else {
        subscriber.with(stderr_layer).init();
        return None;
    };

    subscriber
        .with(fmt::layer().with_writer(writer).with_ansi(false))
        .with(stderr_layer)
        .init();
    tracing::info!(directory = %directory.display(), "logging to file");
    Some(guard)
}

/// `$XDG_STATE_HOME/colorpickle/log`, conventionally `~/.local/state/colorpickle/log`.
fn directory() -> Option<PathBuf> {
    Some(state::directory()?.join(LOG_DIRECTORY))
}

fn appender(directory: &Path) -> Result<(NonBlocking, WorkerGuard)> {
    let file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(LOG_FILE)
        .max_log_files(RETAINED_LOG_FILES)
        .build(directory)
        .with_context(|| format!("failed to open a log file in {}", directory.display()))?;
    Ok(tracing_appender::non_blocking(file))
}

#[cfg(test)]
mod tests {
    use super::{RETAINED_LOG_FILES, appender};
    use std::path::{Path, PathBuf};

    fn temporary_directory(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("colorpickle-log-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    fn log_files(directory: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn writes_a_dated_log_file() {
        let directory = temporary_directory("dated");
        let (_, guard) = appender(&directory).unwrap();
        drop(guard);

        let names = log_files(&directory);
        assert_eq!(names.len(), 1);
        assert!(names[0].starts_with("colorpickle.log."), "{names:?}");

        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn keeps_at_most_the_retained_files() {
        let directory = temporary_directory("retained");
        for day in 1..=RETAINED_LOG_FILES + 3 {
            let stale = directory.join(format!("colorpickle.log.2026-01-{day:02}"));
            std::fs::write(stale, "").unwrap();
        }

        let (_, guard) = appender(&directory).unwrap();
        drop(guard);

        assert!(log_files(&directory).len() <= RETAINED_LOG_FILES);

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
