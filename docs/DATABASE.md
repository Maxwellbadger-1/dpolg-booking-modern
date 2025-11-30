# PostgreSQL Database Setup

## Oracle Cloud Server

**Connection:**
- Host: `141.147.3.123`
- Port: `6432` (pgBouncer - EMPFOHLEN)
- Port: `5432` (Direct - nur Admin)
- Database: `dpolg_booking`
- User: `dpolg_admin`
- Password: `DPolG2025SecureBooking`

**Connection String:**
```
postgres://dpolg_admin:DPolG2025SecureBooking@141.147.3.123:6432/dpolg_booking
```

---

## SSH Access

```bash
ssh -i ~/Downloads/ssh-key-2025-11-14.key ubuntu@141.147.3.123
```

---

## Server-Komponenten

| Komponente | Version | Port |
|------------|---------|------|
| PostgreSQL | 16.11 | 5432 |
| pgBouncer | 1.25.0 | 6432 |
| Ubuntu | 22.04.5 LTS | - |

**pgBouncer Config:**
- Pool Mode: transaction
- Max Connections: 100
- Pool Size: 20

---

## Backend Architektur

**Repository Pattern:** `src-tauri/src/database_pg/`
- `pool.rs` - Connection Pooling
- `error.rs` - Error Handling
- `models.rs` - Rust Structs
- `repositories/` - CRUD Operations

**Fertige Repositories:**
- room_repository.rs (6 Methods)
- guest_repository.rs (8 Methods)
- booking_repository.rs (11 Methods)
- additional_service_repository.rs (7 Methods)
- discount_repository.rs (7 Methods)

---

## Tabellen (23 total)

- rooms (10), guests (257), bookings (323)
- additional_services (392), discounts (185)
- accompanying_guests (52), email_logs (448)
- service_templates, discount_templates, payment_recipients
- etc.

**Migration Status:** SQLite → PostgreSQL ✅ (1,740 rows)
