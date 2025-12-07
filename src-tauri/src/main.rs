// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![feature(coverage_attribute)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[coverage(off)]
fn main() {
    formation_docs_lib::run()
}
