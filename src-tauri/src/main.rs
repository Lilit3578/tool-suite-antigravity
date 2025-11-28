// Prevents additional console window on Windows in release, DO NOT REMOVE!!\n#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    productivity_widgets_lib::run()
}
