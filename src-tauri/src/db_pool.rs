use deadpool_postgres::{Config, Pool, Runtime, ManagerConfig, RecyclingMethod};
use tokio_postgres::NoTls;
use std::env;

/// Creates a PostgreSQL connection pool for multi-user access
///
/// Configuration:
/// - Host: 141.147.3.123 (Oracle Cloud)
/// - Port: 6432 (pgBouncer - Connection Pooling)
/// - Database: dpolg_booking
/// - Pool Size: 20 connections (matches pgBouncer default_pool_size)
pub fn create_pool() -> Result<Pool, Box<dyn std::error::Error>> {
    let mut cfg = Config::new();

    // Oracle Cloud PostgreSQL Server (via pgBouncer)
    cfg.host = Some("141.147.3.123".to_string());
    cfg.port = Some(6432); // pgBouncer port (not 5432!)
    cfg.dbname = Some("dpolg_booking".to_string());
    cfg.user = Some("dpolg_admin".to_string());
    cfg.password = Some("DPolG2025SecureBooking".to_string());

    // Pool configuration
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    // Application name for debugging
    cfg.application_name = Some("dpolg-booking-app".to_string());

    // Create pool (max 20 connections - matches pgBouncer config)
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    println!("✅ PostgreSQL connection pool created");
    println!("   Host: 141.147.3.123:6432 (pgBouncer)");
    println!("   Database: dpolg_booking");

    Ok(pool)
}

/// Test connection to PostgreSQL
pub async fn test_connection(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;

    let row = client
        .query_one("SELECT version()", &[])
        .await?;

    let version: String = row.get(0);
    println!("✅ PostgreSQL connection test successful");
    println!("   Version: {}", version);

    Ok(())
}
