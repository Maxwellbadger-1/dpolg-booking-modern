use deadpool_postgres::{Config, Pool, Runtime, ManagerConfig, RecyclingMethod, PoolConfig, Timeouts};
use tokio_postgres::NoTls;
use std::time::Duration;

pub type DbPool = Pool;

/// Creates PostgreSQL connection pool with optimal settings for multi-user
///
/// Configuration (2025 Best Practices):
/// - Connection pooling via deadpool-postgres
/// - Fast connection recycling with health checks
/// - Direct PostgreSQL connection (no pgBouncer for now)
/// - Auto-reconnect on connection loss
pub fn create_pool() -> Result<Pool, Box<dyn std::error::Error>> {
    let mut cfg = Config::new();

    // Production PostgreSQL Server (Oracle Cloud - DIRECT connection)
    cfg.host = Some("141.147.3.123".to_string());
    cfg.port = Some(5432); // Direct PostgreSQL port (not pgBouncer)
    cfg.dbname = Some("dpolg_booking".to_string());
    cfg.user = Some("dpolg_admin".to_string());
    cfg.password = Some("DPolG2025SecureBooking".to_string());

    // Manager configuration with connection recycling
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    // Pool configuration with timeouts to prevent stale connections
    cfg.pool = Some(PoolConfig {
        max_size: 20,
        timeouts: Timeouts {
            wait: Some(Duration::from_secs(30)),    // Wait for connection
            create: Some(Duration::from_secs(10)),  // Create new connection
            recycle: Some(Duration::from_secs(5)),  // Recycle connection
        },
        ..Default::default()
    });

    // Application name (visible in pg_stat_activity)
    cfg.application_name = Some("dpolg-booking-tauri".to_string());

    // Connection timeout
    cfg.connect_timeout = Some(Duration::from_secs(10));

    // Keepalives to prevent server from closing idle connections
    cfg.keepalives = Some(true);
    cfg.keepalives_idle = Some(Duration::from_secs(30));

    // Create pool with Tokio runtime for proper async handling
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    println!("✅ PostgreSQL connection pool created");
    println!("   Host: 141.147.3.123:5432 (Direct PostgreSQL)");
    println!("   Database: dpolg_booking");
    println!("   Max connections: 20");
    println!("   Keepalives: enabled (30s idle)");
    println!("   TLS: Disabled (testing mode)");

    Ok(pool)
}
