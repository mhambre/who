fn parse_port_argument() -> Option<u16> {
    who::warn!(
        rustc().compare(">=1.88"),
        "Rust 1.88 stabilized let chains for edition 2024; see rust-lang/rust#53667 and rust-lang/rust#132833, then replace this nested `if let` workaround when this crate moves to edition 2024"
    );

    if let Some(raw) = std::env::args().nth(1) {
        if let Ok(port) = raw.parse::<u16>() {
            if port > 0 {
                return Some(port);
            }
        }
    }

    None
}

fn main() {
    match parse_port_argument() {
        Some(port) => println!("selected port: {port}"),
        None => println!("selected port: 3000"),
    }
}
