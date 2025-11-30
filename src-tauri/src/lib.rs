// DPolG Booking System - Modern PostgreSQL Version (2025)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// PostgreSQL version (modern architecture)
pub mod lib_pg;
pub mod config;
pub mod database_pg;
pub mod turso_sync;
pub mod cleaning_timeline_pdf;
