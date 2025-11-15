# 🚀 Development & Deployment Workflow Guide

**Modern Best Practices 2025** - HMR + GitHub CI/CD + PostgreSQL

---

## 📋 Table of Contents

1. [Initial Setup](#initial-setup)
2. [Development Workflow](#development-workflow)
3. [Production Deployment](#production-deployment)
4. [Troubleshooting](#troubleshooting)

---

## 🔧 Initial Setup (One-Time)

### 1. Clone Repository

```bash
git clone https://github.com/YOUR_USERNAME/dpolg-booking-modern.git
cd dpolg-booking-modern
```

### 2. Install Dependencies

```bash
# Frontend dependencies
npm install

# Rust dependencies (automatic via Tauri)
cd src-tauri && cargo build
```

### 3. Environment Configuration

Create `.env` file in project root (NEVER commit this!):

```bash
# Copy template
cp .env.example .env

# Edit with your values
ENVIRONMENT=dev
DATABASE_HOST=141.147.3.123
DATABASE_PORT=6432
DATABASE_NAME=dpolg_booking
DATABASE_USER=dpolg_admin
DATABASE_PASSWORD=DPolG2025SecureBooking
DATABASE_MAX_CONNECTIONS=20
```

**IMPORTANT:** `.env` is already in `.gitignore` - NEVER commit it!

### 4. Verify PostgreSQL Connection

```bash
# Test connection manually (optional)
psql -h 141.147.3.123 -p 6432 -U dpolg_admin -d dpolg_booking

# Or let the app test it automatically on first run
npm run tauri:dev
```

---

## 💻 Development Workflow

### Daily Development Cycle

```bash
# 1. Start development server (ONE COMMAND!)
npm run tauri:dev

# What happens automatically:
# ✅ Kills old processes on ports 1420/1421
# ✅ Starts Vite dev server (port 1420)
# ✅ Starts Tauri with Rust backend
# ✅ Opens app window
# ✅ Enables Hot Module Reloading (HMR)
```

### Making Changes

#### Frontend Changes (React/TypeScript)

```bash
# 1. Edit any .tsx/.ts file in src/
# 2. Save file
# 3. ✨ Browser auto-reloads INSTANTLY (Vite HMR)
# 4. No restart needed!
```

**Example:**
```typescript
// src/components/BookingList.tsx
export function BookingList() {
  return (
    <div className="p-4">  {/* Change padding */}
      <h1>Bookings</h1>
    </div>
  );
}
// Save → Auto-reload in <1 second!
```

#### Backend Changes (Rust)

```bash
# 1. Edit any .rs file in src-tauri/src/
# 2. Save file
# 3. ⚙️  Tauri auto-recompiles Rust code (~5-10 seconds)
# 4. App restarts automatically
```

**Example:**
```rust
// src-tauri/src/lib_pg.rs
#[tauri::command]
async fn get_all_rooms_pg(pool: State<'_, DbPool>) -> Result<Vec<Room>, String> {
    println!("🏠 Fetching all rooms...");  // Add debug log
    RoomRepository::get_all(&pool).await
        .map_err(|e| e.to_string())
}
// Save → Recompile → Restart (~10s)
```

### Testing Changes

#### 1. Manual Testing (Primary)

```bash
# App is already running from `npm run tauri:dev`
# → Test features in the app window
# → Check DevTools Console (Cmd+Option+I / F12)
```

#### 2. Unit Tests (Rust)

```bash
cd src-tauri
cargo test

# Run specific test
cargo test test_room_repository

# With output
cargo test -- --nocapture
```

#### 3. Integration Tests (Playwright)

```bash
# Run all tests
npx playwright test

# Run critical regression tests
npx playwright test tests/critical-regression.spec.ts

# With UI
npx playwright test --ui
```

### Committing Changes

```bash
# 1. Check status
git status

# 2. Stage changes
git add src/components/BookingList.tsx
git add src-tauri/src/lib_pg.rs

# 3. Commit with semantic message
git commit -m "feat(bookings): Add debug logging to room fetching"

# 4. Push to GitHub
git push origin main
```

---

## 🚀 Production Deployment

### GitHub Actions Automated Release

#### Prerequisites (One-Time Setup)

1. **Configure GitHub Secrets** (see [GITHUB_SECRETS_SETUP.md](GITHUB_SECRETS_SETUP.md))
   - `DB_HOST`, `DB_PORT`, `DB_NAME`, `DB_USER`, `DB_PASSWORD`
   - `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

2. **Verify Workflow File** (already configured)
   - `.github/workflows/release.yml`

#### Release Process (Simple!)

```bash
# 1. Ensure all changes are committed
git status  # Should show "nothing to commit"

# 2. Update version in all files (3 locations)
# - package.json
# - src-tauri/Cargo.toml
# - src-tauri/tauri.conf.json

# 3. Commit version bump
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: Bump version to 2.0.0"

# 4. Create Git tag
git tag v2.0.0

# 5. Push to GitHub (triggers CI/CD!)
git push origin main --tags
```

#### What Happens Automatically

```
GitHub Actions Workflow:
┌─────────────────────────────────────────────┐
│ 1. Checkout code                             │
│ 2. Setup Node.js 18                          │
│ 3. Setup Rust stable                         │
│ 4. Install dependencies                      │
│ 5. Inject environment variables from secrets │
│    → ENVIRONMENT=production                  │
│    → DATABASE_HOST=141.147.3.123             │
│    → DATABASE_PASSWORD=***                   │
│ 6. Build Tauri app (Windows/Mac/Linux)       │
│    → Compiles Rust with PostgreSQL config    │
│    → Signs binaries with private key         │
│ 7. Create GitHub Release                     │
│ 8. Upload installers                         │
│    → Windows: .msi + .exe                    │
│    → macOS: .dmg + .app                      │
│    → Linux: .deb + .AppImage                 │
│ 9. Generate updater JSON for auto-updates   │
└─────────────────────────────────────────────┘
```

#### Monitoring Build Progress

```bash
# 1. Go to GitHub Actions tab
https://github.com/YOUR_USERNAME/dpolg-booking-modern/actions

# 2. Find your workflow run (tagged with version)
# 3. Watch real-time logs
# 4. Wait for green checkmark ✅ (~15-20 minutes)

# 5. Download artifacts or check Releases
https://github.com/YOUR_USERNAME/dpolg-booking-modern/releases
```

---

## 🛠️ Advanced Workflows

### Feature Branch Workflow (Recommended)

```bash
# 1. Create feature branch
git checkout -b feature/new-booking-ui

# 2. Make changes
# ... edit files ...

# 3. Test locally
npm run tauri:dev

# 4. Run tests
npx playwright test tests/critical-regression.spec.ts

# 5. Commit to feature branch
git add .
git commit -m "feat: Redesign booking UI with better UX"
git push origin feature/new-booking-ui

# 6. Create Pull Request on GitHub
# 7. Review + Merge to main
# 8. Create release tag (see Release Process above)
```

### Hot Reload Optimization

**Problem:** Slow reloads?

**Solutions:**

1. **Reduce watch scope** (already configured in `vite.config.ts`):
   ```typescript
   watch: {
     ignored: [
       "**/src-tauri/**",  // Don't watch Rust files
       "**/*.db",          // Don't watch database
     ]
   }
   ```

2. **Use incremental compilation** (already configured in `Cargo.toml`):
   ```toml
   [profile.dev]
   opt-level = 1
   codegen-units = 256  // Faster parallel compilation
   ```

3. **Clear Rust cache** (if builds become slow):
   ```bash
   cd src-tauri
   cargo clean
   cd ..
   npm run tauri:dev
   ```

### Database Changes During Development

**Adding new columns/tables:**

```rust
// 1. Update PostgreSQL schema (via SSH or DBeaver)
ALTER TABLE bookings ADD COLUMN notes TEXT;

// 2. Update Rust model
#[derive(Serialize, Deserialize)]
pub struct Booking {
    pub id: i32,
    // ... existing fields ...
    pub notes: Option<String>,  // ← Add new field
}

// 3. Update repository query
pub async fn get_all(pool: &DbPool) -> DbResult<Vec<Booking>> {
    let rows = client.query(
        "SELECT id, ..., notes FROM bookings",  // ← Add to SELECT
        &[],
    ).await?;
    // ...
}

// 4. Save → Auto-reload
```

---

## 🔍 Troubleshooting

### Port Already in Use

**Error:** `Port 1420 is already in use`

**Fix:**
```bash
# Kill port manually (script does this automatically)
npx kill-port 1420 1421

# Or restart the dev server
npm run tauri:dev  # Script kills ports first
```

### Rust Compilation Errors

**Error:** `error: could not compile...`

**Fixes:**

1. **Clear build cache:**
   ```bash
   cd src-tauri
   cargo clean
   cargo build
   ```

2. **Update dependencies:**
   ```bash
   cargo update
   ```

3. **Check Rust version:**
   ```bash
   rustc --version  # Should be 1.70+
   rustup update
   ```

### Database Connection Failed

**Error:** `Connection refused` or `Password authentication failed`

**Fixes:**

1. **Verify .env file exists:**
   ```bash
   cat .env  # Should show DATABASE_* variables
   ```

2. **Test connection manually:**
   ```bash
   psql -h 141.147.3.123 -p 6432 -U dpolg_admin -d dpolg_booking
   # Enter password: DPolG2025SecureBooking
   ```

3. **Check Oracle Cloud firewall:**
   - Port 6432 must be open in Security List
   - Ubuntu iptables must allow port 6432

### GitHub Actions Build Failed

**Error:** Build fails on GitHub but works locally

**Common Causes:**

1. **Missing GitHub Secrets:**
   ```bash
   # Check secrets are set at:
   https://github.com/YOUR_USERNAME/dpolg-booking-modern/settings/secrets/actions
   ```

2. **Environment variable mismatch:**
   ```bash
   # Ensure secret names match EXACTLY (case-sensitive):
   DB_HOST (not db_host or DATABASE_HOST)
   ```

3. **Signing key issues:**
   ```bash
   # Verify TAURI_SIGNING_PRIVATE_KEY is set
   # Verify key format is correct (should be base64)
   ```

### HMR Not Working

**Problem:** Changes don't auto-reload

**Fixes:**

1. **Check Vite dev server is running:**
   ```bash
   # Should see:
   # VITE v7.x.x ready in XXX ms
   # ➜ Local: http://localhost:1420/
   ```

2. **Check browser console for errors:**
   ```javascript
   // Should see WebSocket connection:
   [vite] connected.
   ```

3. **Restart dev server:**
   ```bash
   # Stop with Ctrl+C
   npm run tauri:dev
   ```

---

## 📊 Performance Benchmarks

### Development Mode (HMR Enabled)

| Operation | Time | Notes |
|-----------|------|-------|
| **Initial Startup** | ~10s | Rust compilation + Vite |
| **Frontend Hot Reload** | <1s | React component changes |
| **Rust Recompile** | ~5-10s | Backend changes |
| **Full Restart** | ~10s | After config changes |

### Production Build (GitHub Actions)

| Platform | Build Time | Binary Size |
|----------|-----------|-------------|
| **Windows** | ~12 min | ~15 MB (.msi) |
| **macOS** | ~15 min | ~20 MB (.dmg) |
| **Linux** | ~10 min | ~18 MB (.AppImage) |

---

## 🎯 Best Practices Summary

### ✅ DO:

- **Use `npm run tauri:dev`** for all development (HMR enabled)
- **Commit often** with semantic messages
- **Test locally** before pushing
- **Use feature branches** for bigger changes
- **Run regression tests** before releases
- **Use GitHub Secrets** for production credentials
- **Update version** in all 3 files before tagging

### ❌ DON'T:

- **NEVER** commit `.env` file
- **NEVER** use `npm run tauri dev` directly (no port cleanup!)
- **NEVER** commit hardcoded passwords
- **NEVER** skip regression tests
- **NEVER** merge untested code to main
- **NEVER** forget to tag releases

---

## 📚 Quick Reference

### Essential Commands

```bash
# Development
npm run tauri:dev           # Start dev server with HMR
npm run build               # Build frontend only
npm run tauri:build         # Build production app locally

# Testing
cargo test                  # Rust unit tests
npx playwright test         # Integration tests
npm run type-check          # TypeScript type check

# Git
git status                  # Check changes
git add .                   # Stage all changes
git commit -m "msg"         # Commit with message
git push                    # Push to GitHub
git tag v2.0.0              # Create version tag
git push --tags             # Push tags

# Database
psql -h 141.147.3.123 -p 6432 -U dpolg_admin -d dpolg_booking  # Connect
\dt                         # List tables
\d bookings                 # Describe table
SELECT * FROM rooms;        # Query data
```

### File Locations

```
.env                        # Local environment config (NOT in Git)
.env.example                # Environment template (in Git)
.github/workflows/release.yml  # CI/CD pipeline
src-tauri/src/config.rs     # Environment configuration
src-tauri/src/database_pg/  # PostgreSQL layer
src-tauri/src/lib_pg.rs     # PostgreSQL commands
src-tauri/Cargo.toml        # Rust dependencies
package.json                # Node dependencies + scripts
vite.config.ts              # Vite HMR configuration
```

---

**Created:** 2025-11-14
**Last Updated:** 2025-11-14
**Status:** 🟢 Production Ready
