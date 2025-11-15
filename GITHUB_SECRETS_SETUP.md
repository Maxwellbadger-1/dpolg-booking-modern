# 🔐 GitHub Secrets Setup Guide

**Purpose:** Configure GitHub Secrets for automated production builds with PostgreSQL

---

## 📋 Required Secrets

Navigate to: **Repository Settings → Secrets and variables → Actions → New repository secret**

### 1. Database Configuration (PostgreSQL)

Add these secrets for PostgreSQL production connection:

| Secret Name | Value | Description |
|------------|-------|-------------|
| `DB_HOST` | `141.147.3.123` | Oracle Cloud PostgreSQL server IP |
| `DB_PORT` | `6432` | pgBouncer port (transaction pooling) |
| `DB_NAME` | `dpolg_booking` | Database name |
| `DB_USER` | `dpolg_admin` | Database user |
| `DB_PASSWORD` | `DPolG2025SecureBooking` | Database password |

### 2. Tauri Code Signing (Already Configured)

These should already exist from previous releases:

| Secret Name | Description |
|------------|-------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Private key for code signing (.key file content) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for private key (if passwordless, leave empty) |

---

## 🚀 How It Works

### Development Workflow (Local Machine)

```bash
# 1. Uses .env file for database config
ENVIRONMENT=dev
DATABASE_HOST=141.147.3.123
DATABASE_PORT=6432
DATABASE_USER=dpolg_admin
DATABASE_PASSWORD=DPolG2025SecureBooking

# 2. Start dev server with HMR
npm run tauri:dev

# 3. Frontend changes → instant reload (Vite HMR)
# 4. Backend changes → auto-recompile (Tauri watch)
```

### Production Workflow (GitHub Actions)

```bash
# 1. Commit and push changes
git add .
git commit -m "feat: Add new feature"
git push origin main

# 2. Create version tag
git tag v2.0.0
git push --tags

# 3. GitHub Actions automatically:
#    - Builds for Windows, Mac, Linux
#    - Injects secrets as environment variables
#    - Compiles Rust with ENVIRONMENT=production
#    - Signs binaries with private key
#    - Creates GitHub Release
#    - Uploads installers
```

---

## 🔧 Configuration Flow

### GitHub Actions → Environment Variables → Rust Config

```yaml
# .github/workflows/release.yml
env:
  ENVIRONMENT: production              # ← Triggers production mode
  DATABASE_HOST: ${{ secrets.DB_HOST }}
  DATABASE_PASSWORD: ${{ secrets.DB_PASSWORD }}
  # ... other secrets
```

↓

```rust
// src-tauri/src/config.rs
impl AppConfig {
    pub fn from_env() -> Self {
        let environment = env::var("ENVIRONMENT")  // ← Reads from GitHub
            .unwrap_or_else(|_| "dev".to_string());

        let password = env::var("DATABASE_PASSWORD")  // ← Reads secret
            .unwrap_or_else(|_| {
                if environment == "production" {
                    panic!("DATABASE_PASSWORD must be set!");  // ← Fails if missing
                }
                "DPolG2025SecureBooking".to_string()
            });
    }
}
```

↓

```rust
// src-tauri/src/database_pg/pool.rs
pub fn create_pool() -> Result<Pool> {
    let config = AppConfig::from_env();  // ← Uses environment

    cfg.host = Some(config.database.host);     // ← From secret
    cfg.password = Some(config.database.password);
    // ...
}
```

---

## ✅ Verification Steps

### Step 1: Add Secrets

1. Go to: `https://github.com/YOUR_USERNAME/dpolg-booking-modern/settings/secrets/actions`
2. Click **New repository secret**
3. Add each secret from the table above
4. Click **Add secret**

### Step 2: Verify Secrets Are Set

After adding all secrets, you should see:

```
DB_HOST          Updated 1 minute ago
DB_PORT          Updated 1 minute ago
DB_NAME          Updated 1 minute ago
DB_USER          Updated 1 minute ago
DB_PASSWORD      Updated 1 minute ago
TAURI_SIGNING_PRIVATE_KEY          Updated X days ago
TAURI_SIGNING_PRIVATE_KEY_PASSWORD Updated X days ago
```

### Step 3: Test Build

```bash
# Trigger a test build
git tag v2.0.0-test
git push --tags

# Check GitHub Actions logs
# → Should see "ENVIRONMENT: production"
# → Should see "Connected to PostgreSQL"
# → Should NOT see any plaintext passwords!
```

---

## 🚨 Security Best Practices

### ✅ DO:

- **Use GitHub Secrets** for all sensitive data (passwords, keys)
- **Use `.env.example`** as template (checked into Git)
- **Use `.env`** for local development (NOT in Git!)
- **Verify secrets** are injected correctly in Actions logs

### ❌ DON'T:

- **NEVER** commit `.env` file to Git (already in `.gitignore`)
- **NEVER** commit passwords/keys directly in code
- **NEVER** use `console.log()` or `println!()` with secrets
- **NEVER** share secrets in Slack/Email/etc.

---

## 🔍 Troubleshooting

### Problem: "DATABASE_PASSWORD must be set in production!"

**Cause:** GitHub Secret `DB_PASSWORD` is missing or misspelled

**Fix:**
1. Check secret name is exactly `DB_PASSWORD` (case-sensitive!)
2. Verify secret has a value (not empty)
3. Re-run workflow

### Problem: "Connection refused" in production build

**Cause:** Database host/port incorrect

**Fix:**
1. Verify `DB_HOST` = `141.147.3.123`
2. Verify `DB_PORT` = `6432` (pgBouncer, NOT 5432!)
3. Test connection manually:
   ```bash
   psql -h 141.147.3.123 -p 6432 -U dpolg_admin -d dpolg_booking
   ```

### Problem: Build fails but secrets are correct

**Cause:** Rust compilation issue or signing problem

**Fix:**
1. Check GitHub Actions logs for exact error
2. Test local build first: `npm run tauri build`
3. Verify `TAURI_SIGNING_PRIVATE_KEY` is not corrupted

---

## 📚 References

- [GitHub Encrypted Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Tauri Environment Variables](https://tauri.app/v1/guides/building/environment-variables)
- [PostgreSQL Connection Security](https://www.postgresql.org/docs/16/libpq-connect.html)

---

**Created:** 2025-11-14
**Last Updated:** 2025-11-14
**Status:** 🟢 Ready for Production
