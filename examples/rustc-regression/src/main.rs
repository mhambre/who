use std::hint::black_box;

#[repr(u16)]
#[derive(Clone, Copy, Debug)]
enum Checksum {
    X(bool, u64),
    Y(u64, u64),
}

fn finalize(checksum: Checksum) -> u64 {
    match checksum {
        Checksum::X(false, sum) => sum,
        Checksum::X(true, sum) => sum,
        Checksum::Y(prefix, sum) => prefix.saturating_sub(prefix) + sum,
    }
}

fn run_with_workaround(checksum: Option<Checksum>) -> Option<u64> {
    who::warn!(
        rustc().compare(">=1.97.1"),
        "Rust 1.97.1 fixed rust-lang/rust#159035; recheck whether the manual match can be simplified back to Option::map"
    );

    match checksum {
        Some(checksum) => Some(finalize(checksum)),
        None => None,
    }
}

fn main() {
    let none = run_with_workaround(black_box(None));
    let x = run_with_workaround(black_box(Some(Checksum::X(true, 7))));
    let y = run_with_workaround(black_box(Some(Checksum::Y(11, 42))));

    println!("None -> {none:?}");
    println!("X -> {x:?}");
    println!("Y -> {y:?}");
}
