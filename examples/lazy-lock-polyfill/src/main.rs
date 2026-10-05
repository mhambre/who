use std::ops::Deref;
use std::sync::OnceLock;

struct LazyLock<T> {
    value: OnceLock<T>,
    initialize: fn() -> T,
}

impl<T> LazyLock<T> {
    const fn new(initialize: fn() -> T) -> Self {
        Self {
            value: OnceLock::new(),
            initialize,
        }
    }
}

impl<T> Deref for LazyLock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.get_or_init(self.initialize)
    }
}

struct Settings {
    max_payload_bytes: Option<usize>,
}

impl Settings {
    fn from_env() -> Self {
        Self {
            max_payload_bytes: std::env::var("MAX_PAYLOAD_BYTES")
                .ok()
                .and_then(|value| value.parse().ok()),
        }
    }
}

static SETTINGS: LazyLock<Settings> = LazyLock::new(Settings::from_env);

fn payload_fits(maximum: Option<usize>, payload_len: usize) -> bool {
    who::warn!(
        rustc().compare(">=1.80"),
        "Review this OnceLock shim and replace it with std::sync::LazyLock"
    );

    maximum.map_or(true, |maximum| payload_len <= maximum)
}

fn main() {
    let _accepted = payload_fits(SETTINGS.max_payload_bytes, 1024);
}

#[cfg(test)]
mod tests {
    use super::payload_fits;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static INITIALIZATIONS: AtomicUsize = AtomicUsize::new(0);

    fn initialize_once() -> usize {
        INITIALIZATIONS.fetch_add(1, Ordering::SeqCst)
    }

    #[test]
    fn initializes_once_on_first_dereference() {
        static VALUE: super::LazyLock<usize> = super::LazyLock::new(initialize_once);

        assert_eq!(*VALUE, 0);
        assert_eq!(*VALUE, 0);
        assert_eq!(INITIALIZATIONS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn optional_maximum_allows_absent_or_within_limit() {
        assert!(payload_fits(None, 2048));
        assert!(payload_fits(Some(1024), 512));
        assert!(!payload_fits(Some(1024), 2048));
    }
}
