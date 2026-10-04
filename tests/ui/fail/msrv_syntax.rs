fn main() {
    who::error!(msrv("caller").compare(">=1"), "no receiver arguments");
    who::error!(msrv().compare("banana"), "bad requirement");
    who::error!(msrv().changed_from("1.74"), "bad exact version");
    who::error!(msrv().after("1.74.0"), "unsupported method");
}
