use std::fmt;

/// Custom error type for database operations (2025 Best Practice)
///
/// Benefits:
/// - Type-safe error handling
/// - Easy conversion to Tauri command errors
/// - Detailed error information for debugging
/// - User-friendly error messages for frontend
#[derive(Debug)]
pub enum DbError {
    /// PostgreSQL query error
    QueryError(String),
    /// Connection pool error
    PoolError(String),
    /// Connection error
    ConnectionError(String),
    /// Record not found
    NotFound(String),
    /// Validation error
    ValidationError(String),
    /// Constraint violation (e.g., unique, foreign key)
    ConstraintViolation(String),
    /// Optimistic locking conflict - record was modified by another user
    ConflictError(String),
    /// Double booking conflict - room is already booked for the requested period
    DoubleBookingError(String),
    /// General database error
    Other(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::QueryError(e) => write!(f, "Database query error: {}", e),
            DbError::PoolError(e) => write!(f, "Connection pool error: {}", e),
            DbError::ConnectionError(e) => write!(f, "Connection error: {}", e),
            DbError::NotFound(msg) => write!(f, "Record not found: {}", msg),
            DbError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            DbError::ConstraintViolation(msg) => write!(f, "Constraint violation: {}", msg),
            DbError::ConflictError(msg) => write!(f, "Conflict: {}", msg),
            DbError::DoubleBookingError(msg) => write!(f, "Doppelbuchung: {}", msg),
            DbError::Other(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for DbError {}

// Conversions from various error types
impl From<tokio_postgres::Error> for DbError {
    fn from(err: tokio_postgres::Error) -> Self {
        // Log the FULL error for debugging
        eprintln!("🔴 PostgreSQL Error: {:?}", err);

        // Check for specific error types
        if let Some(code) = err.code() {
            eprintln!("   SQL Code: {}", code.code());
            match code.code() {
                "23505" => DbError::ConstraintViolation("Unique constraint violated".to_string()),
                "23503" => DbError::ConstraintViolation("Foreign key constraint violated".to_string()),
                _ => DbError::QueryError(format!("{:?}", err)),
            }
        } else {
            DbError::QueryError(format!("{:?}", err))
        }
    }
}

impl From<deadpool_postgres::PoolError> for DbError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        DbError::PoolError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for DbError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        DbError::Other(err.to_string())
    }
}

// Conversion to String for Tauri commands
impl From<DbError> for String {
    fn from(err: DbError) -> Self {
        err.to_string()
    }
}

/// Result type for database operations
pub type DbResult<T> = Result<T, DbError>;
