#![deny(warnings)]

who::error!(!path(std::sync::OnceLock).exists(), "importable type");
who::error!(!path(core::marker::Send).exists(), "importable trait");
who::error!(!path(alloc::vec).exists(), "importable module and macro");
who::error!(!path(proc_macro::TokenStream).exists(), "importable proc-macro type");
who::error!(!path(std::println).exists(), "importable macro");
who::error!(!path(std).exists(), "sysroot crate itself");
who::error!(!path(::core::mem).exists(), "absolute path");
who::error!(!path(r#std::mem).exists(), "raw root identifier");
who::error!(!path(core::usize::MAX).exists(), "importable constant");

#[cfg(windows)]
who::error!(!path(std::os::windows).exists(), "Windows target API");
#[cfg(not(windows))]
who::error!(path(std::os::windows).exists(), "no Windows target API");
#[cfg(target_arch = "aarch64")]
who::error!(!path(core::arch::aarch64).exists(), "ARM target API");
#[cfg(target_arch = "x86_64")]
who::error!(!path(core::arch::x86_64).exists(), "x86 target API");

fn main() {
    who::warn!(path(std::who_definitely_missing::Item).exists(), "inactive warning");
    who::error!(
        !(path(core::mem::size_of).exists() && !path(core::who_definitely_missing).exists())
            || path(std::who_definitely_missing).exists(),
        "boolean composition"
    );
    who::error!(path(std::sync::LazyLock).exists() && rustc().compare("<1.80")
        || !path(std::sync::LazyLock).exists() && rustc().compare(">=1.80"),
        "LazyLock availability follows the active toolchain");
}
