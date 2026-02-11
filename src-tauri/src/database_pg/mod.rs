// Modern PostgreSQL Database Layer (2025 Best Practices)
// Architecture: Repository Pattern + Type-safe Queries + Error Handling + Real-Time Events

pub mod pool;
pub mod error;
pub mod models;
pub mod repositories;
pub mod queries;
pub mod listener;  // ← NEW: Real-Time LISTEN/NOTIFY support

// Re-exports for convenience
pub use pool::{create_pool, DbPool};
pub use error::{DbError, DbResult};
pub use models::*;
pub use repositories::*;
pub use listener::{DbChangeEvent, start_pg_listener};  // Real-Time LISTEN/NOTIFY

/// Initialize PostgreSQL connection pool and verify connection
pub async fn init_database() -> DbResult<DbPool> {
    println!("🔄 Creating PostgreSQL connection pool...");
    let pool = pool::create_pool()?;

    println!("🔄 Testing connection with query...");
    // Test connection
    let client = pool.get().await.map_err(|e| {
        eprintln!("❌ Failed to get connection from pool: {}", e);
        DbError::ConnectionError(format!("Pool get failed: {}", e))
    })?;

    println!("🔄 Executing SELECT version()...");
    let row = client
        .query_one("SELECT version()", &[])
        .await
        .map_err(|e| {
            eprintln!("❌ Query failed: {}", e);
            DbError::QueryError(format!("SELECT version() failed: {}", e))
        })?;

    let version: String = row.get(0);

    println!("✅ Connected to PostgreSQL");
    println!("   {}", version);

    Ok(pool)
}
