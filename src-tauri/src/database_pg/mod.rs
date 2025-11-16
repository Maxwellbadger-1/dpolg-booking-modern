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
pub use listener::{DbChangeEvent, EventSender, EventReceiver, start_listener_background};  // ← NEW

/// Initialize PostgreSQL connection pool and verify connection
pub async fn init_database() -> DbResult<DbPool> {
    let pool = pool::create_pool()?;

    // Test connection
    let client = pool.get().await?;
    let version: String = client
        .query_one("SELECT version()", &[])
        .await?
        .get(0);

    println!("✅ Connected to PostgreSQL");
    println!("   {}", version);

    Ok(pool)
}
