use serde::{Deserialize, Serialize};
use std::env;

/// Application configuration (Environment-based)
///
/// Best Practice 2025:
/// - Development: Uses .env file or defaults
/// - Production: Uses environment variables set by CI/CD
/// - Never commit secrets to Git!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub environment: Environment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

impl AppConfig {
    /// Load configuration from environment variables with fallback to defaults
    ///
    /// Environment variables (set these in production):
    /// - DATABASE_HOST (default: 141.147.3.123)
    /// - DATABASE_PORT (default: 6432)
    /// - DATABASE_NAME (default: dpolg_booking)
    /// - DATABASE_USER (default: dpolg_admin)
    /// - DATABASE_PASSWORD (REQUIRED in production!)
    /// - ENVIRONMENT (dev or prod, default: dev)
    pub fn from_env() -> Self {
        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "dev".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            _ => Environment::Development,
        };

        let database = DatabaseConfig {
            host: env::var("DATABASE_HOST")
                .unwrap_or_else(|_| "141.147.3.123".to_string()),
            port: env::var("DATABASE_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(6432),
            database: env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "dpolg_booking".to_string()),
            username: env::var("DATABASE_USER")
                .unwrap_or_else(|_| "dpolg_admin".to_string()),
            password: env::var("DATABASE_PASSWORD")
                .unwrap_or_else(|_| {
                    if environment == Environment::Production {
                        panic!("DATABASE_PASSWORD must be set in production!");
                    }
                    "DPolG2025SecureBooking".to_string()
                }),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(20),
        };

        println!("✅ Configuration loaded");
        println!("   Environment: {:?}", environment);
        println!("   Database: {}:{}/{}", database.host, database.port, database.database);
        println!("   Max connections: {}", database.max_connections);

        Self {
            database,
            environment,
        }
    }

    /// Get database connection string
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        env::set_var("ENVIRONMENT", "dev");
        env::set_var("DATABASE_HOST", "localhost");
        env::set_var("DATABASE_PORT", "5432");

        let config = AppConfig::from_env();

        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
    }
}
