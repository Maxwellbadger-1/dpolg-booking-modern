// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Use PostgreSQL version (new, modern architecture)
    dpolg_booking_modern_lib::lib_pg::run_pg()

    // Old SQLite version (legacy):
    // dpolg_booking_modern_lib::run()
}
