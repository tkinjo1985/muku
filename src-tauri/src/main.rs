// GUI app — never open a console window on Windows, even in debug builds
#![windows_subsystem = "windows"]

fn main() {
    muku_lib::run()
}
