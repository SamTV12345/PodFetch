//! Running work outside the server's async runtime.
//!
//! Large parts of the import and download pipeline still mix async I/O with
//! blocking calls (diesel, `reqwest::blocking`). Running such work on a worker
//! of the server's runtime would stall request handling – and
//! `reqwest::blocking` even panics when it is used from within an async
//! context. These helpers give that work its own OS thread with its own
//! single-threaded tokio runtime, so the rest of the code has exactly one place
//! that knows about the workaround.

use std::fmt::Display;
use std::future::Future;
use std::thread;

use crate::error::{CustomError, CustomErrorInner, ErrorSeverity};

fn build_runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
}

/// Spawns `make_future` on a dedicated thread and returns immediately.
///
/// The future is created *on* the new thread, so it does not need to be `Send`.
/// A failing job is logged under `job`; nobody waits for the result.
pub fn spawn_detached<F, Fut, E>(job: &'static str, make_future: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), E>>,
    E: Display,
{
    let spawned =
        thread::Builder::new()
            .name(job.to_owned())
            .spawn(move || match build_runtime() {
                Ok(runtime) => {
                    if let Err(error) = runtime.block_on(make_future()) {
                        tracing::error!("Background job '{job}' failed: {error}");
                    }
                }
                Err(error) => tracing::error!("Could not start runtime for '{job}': {error}"),
            });
    if let Err(error) = spawned {
        tracing::error!("Could not spawn thread for '{job}': {error}");
    }
}

/// Runs `make_future` on a dedicated thread and blocks until it finishes.
///
/// Use this to call async code from synchronous code that may itself run
/// inside an async context, where `Handle::block_on` would panic.
pub fn run_isolated<F, Fut, T>(job: &'static str, make_future: F) -> Result<T, CustomError>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, CustomError>>,
    T: Send + 'static,
{
    let unavailable = move |reason: String| -> CustomError {
        CustomErrorInner::Conflict(format!("{job}: {reason}"), ErrorSeverity::Warning).into()
    };

    thread::Builder::new()
        .name(job.to_owned())
        .spawn(move || {
            build_runtime()
                .map_err(|e| unavailable(format!("could not start runtime: {e}")))?
                .block_on(make_future())
        })
        .map_err(|e| unavailable(format!("could not spawn thread: {e}")))?
        .join()
        .unwrap_or_else(|_| Err(unavailable("thread panicked".to_owned())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn run_isolated_returns_the_result() {
        let value = run_isolated("test", || async { Ok::<_, CustomError>(42) }).unwrap();
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn run_isolated_works_inside_an_async_context() {
        let value = run_isolated("test", || async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            Ok::<_, CustomError>("done")
        })
        .unwrap();
        assert_eq!(value, "done");
    }

    #[test]
    fn run_isolated_turns_a_panic_into_an_error() {
        let result: Result<(), _> = run_isolated("test", || async { panic!("boom") });
        assert!(result.is_err());
    }

    #[test]
    fn spawn_detached_runs_the_job() {
        let (tx, rx) = mpsc::channel();
        spawn_detached("test", move || async move {
            tx.send(()).map_err(|e| e.to_string())
        });
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
    }
}
