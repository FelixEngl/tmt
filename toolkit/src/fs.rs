use std::io::ErrorKind;
use std::path::Path;
use itertools::{Itertools, Position};

/// A copy method that allows to retry the copy n times.
pub fn retry_copy<A: AsRef<Path>, B: AsRef<Path>>(a: A, b: B, retries: usize) -> Result<u64, std::io::Error> {
    let a = a.as_ref();
    if !a.exists() {
        return Err(std::io::Error::new(ErrorKind::NotFound, format!("{:?} not found", a)));
    }
    let b = b.as_ref();
    for (marker, ct) in (0..retries).with_position() {
        match std::fs::copy(
            a,
            b,
        ) {
            Ok(value) => {
                return Ok(value)
            }
            Err(err) if matches!(marker, Position::Last | Position::Only) => {
                log::error!("Error ({}/{retries}): {}", ct+1, err);
                return Err(err.into())
            }
            Err(err) => {
                if matches!(err.kind(), ErrorKind::AddrInUse | ErrorKind::ConnectionAborted | ErrorKind::Interrupted | ErrorKind::TimedOut) {
                    log::warn!("Recoverable Error ({}/{retries}): {}", ct+1, err);
                } else {
                    match err.raw_os_error() {
                        Some(59) => {
                            log::warn!("Recoverable network error ({}/10): {}", ct+1, err);
                        }
                        _ => {
                            log::error!("Error ({}/{retries}): {}", ct+1, err);
                            return Err(err.into());
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
    unreachable!()
}