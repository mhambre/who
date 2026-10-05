use std::ops::Deref;
use std::sync::OnceLock;

who::warn!(
    msrv().compare(">=1.80") || path(std::sync::LazyLock).exists(),
    "Rust now provides std::sync::LazyLock; replace compat::LazyLock with the standard type"
);

mod compat {
    use super::{Deref, OnceLock};

    pub struct LazyLock<T> {
        once: OnceLock<T>,
        init: fn() -> T,
    }

    impl<T> LazyLock<T> {
        pub const fn new(init: fn() -> T) -> Self {
            Self {
                once: OnceLock::new(),
                init,
            }
        }

        pub fn force(&self) -> &T {
            self.once.get_or_init(self.init)
        }
    }

    impl<T> Deref for LazyLock<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.force()
        }
    }
}

fn build_message() -> String {
    "hello from a LazyLock polyfill".to_owned()
}

static MESSAGE: compat::LazyLock<String> = compat::LazyLock::new(build_message);

fn main() {
    println!("{}", &*MESSAGE);
}
