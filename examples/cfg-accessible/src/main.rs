#![cfg_attr(feature = "nightly-cfg-accessible", feature(cfg_accessible))]

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

fn greeting() -> String {
    "Hello from the compatibility shim".to_owned()
}

static GREETING: LazyLock<String> = LazyLock::new(greeting);

#[cfg(not(feature = "nightly-cfg-accessible"))]
#[allow(dead_code)]
fn review_when_native_lazy_lock_is_available() {
    who::warn!(
        path(std::sync::LazyLock).exists(),
        "std::sync::LazyLock is available; review and remove this compatibility shim"
    );
}

#[cfg(feature = "nightly-cfg-accessible")]
#[cfg_accessible(std::sync::LazyLock)]
#[allow(dead_code)]
fn review_when_native_lazy_lock_is_available() {
    who::warn!(
        path(std::sync::LazyLock).exists(),
        "std::sync::LazyLock is available; review and remove this compatibility shim"
    );
}

fn main() {
    println!("{}", &*GREETING);
}
