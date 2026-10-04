fn main() {
    who::error!(dependency("syn").compare("*"), "multiple resolved versions");
}
