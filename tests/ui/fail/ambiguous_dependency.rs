fn main() {
    who::error!(dependency("syn").matches("*"), "multiple resolved versions");
}
