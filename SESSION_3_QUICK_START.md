# 🚀 Session 3 - Quick Start Guide

**Ziel:** Email, Reminder, Companion & Template Repositories
**Geschätzte Dauer:** 5-6 Stunden
**Status:** ⏳ Bereit zum Start

---

## 📋 TODO-Liste Session 3

### Priority 1: EmailLogRepository ⭐⭐⭐
- [ ] EmailLog Model in models.rs hinzufügen
- [ ] EmailLogRepository erstellen (8 Methoden)
- [ ] Commands zu lib_pg.rs hinzufügen
- [ ] Registrieren in mod.rs + invoke_handler
- **Daten:** 448 rows
- **Geschätzte Dauer:** 75 Min

### Priority 2: ReminderRepository ⭐⭐
- [ ] Reminder Model bereits vorhanden (prüfen!)
- [ ] ReminderRepository erstellen (7 Methoden)
- [ ] Commands zu lib_pg.rs hinzufügen
- [ ] Registrieren in mod.rs + invoke_handler
- **Daten:** 18 rows
- **Geschätzte Dauer:** 60 Min

### Priority 3: AccompanyingGuestRepository ⭐⭐
- [ ] AccompanyingGuest Model hinzufügen
- [ ] AccompanyingGuestRepository erstellen (6 Methoden)
- [ ] Commands zu lib_pg.rs hinzufügen
- [ ] Registrieren in mod.rs + invoke_handler
- **Daten:** 52 rows
- **Geschätzte Dauer:** 60 Min

### Priority 4: ServiceTemplateRepository ⭐
- [ ] ServiceTemplate Model hinzufügen
- [ ] ServiceTemplateRepository erstellen (6 Methoden)
- [ ] Commands zu lib_pg.rs hinzufügen
- [ ] Registrieren in mod.rs + invoke_handler
- **Geschätzte Dauer:** 60 Min

### Priority 5: DiscountTemplateRepository ⭐
- [ ] DiscountTemplate Model hinzufügen
- [ ] DiscountTemplateRepository erstellen (6 Methoden)
- [ ] Commands zu lib_pg.rs hinzufügen
- [ ] Registrieren in mod.rs + invoke_handler
- **Geschätzte Dauer:** 60 Min

---

## 🔧 Repository Template (Kopieren & Anpassen)

```rust
use crate::database_pg::{DbPool, DbResult, EntityName};

pub struct EntityNameRepository;

impl EntityNameRepository {
    /// Get all entities
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<EntityName>> {
        let client = pool.get().await?;
        let rows = client.query(
            "SELECT id, field1, field2, ... FROM table_name ORDER BY created_at DESC",
            &[],
        ).await?;
        Ok(rows.into_iter().map(EntityName::from).collect())
    }

    /// Get entity by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<EntityName> {
        let client = pool.get().await?;
        let row = client.query_one(
            "SELECT id, field1, field2, ... FROM table_name WHERE id = $1",
            &[&id],
        ).await.map_err(|_| DbError::NotFound(format!("Entity {} not found", id)))?;
        Ok(EntityName::from(row))
    }

    /// Create new entity
    pub async fn create(pool: &DbPool, /* params */) -> DbResult<EntityName> {
        let client = pool.get().await?;
        let row = client.query_one(
            "INSERT INTO table_name (...) VALUES (...) RETURNING *",
            &[/* params */],
        ).await?;
        Ok(EntityName::from(row))
    }

    /// Update entity
    pub async fn update(pool: &DbPool, id: i32, /* params */) -> DbResult<EntityName> {
        let client = pool.get().await?;
        let row = client.query_one(
            "UPDATE table_name SET ... WHERE id = $1 RETURNING *",
            &[&id, /* params */],
        ).await.map_err(|_| DbError::NotFound(format!("Entity {} not found", id)))?;
        Ok(EntityName::from(row))
    }

    /// Delete entity
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;
        let rows = client.execute("DELETE FROM table_name WHERE id = $1", &[&id]).await?;
        if rows == 0 {
            return Err(DbError::NotFound(format!("Entity {} not found", id)));
        }
        Ok(())
    }

    /// Count entities
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;
        let row = client.query_one("SELECT COUNT(*) as count FROM table_name", &[]).await?;
        Ok(row.get("count"))
    }
}
```

---

## 📝 Checkliste pro Repository

### 1. Model hinzufügen (models.rs)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityName {
    pub id: i32,
    // ... fields
}

impl From<Row> for EntityName {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            // ... fields
        }
    }
}
```

### 2. Repository erstellen
- Datei: `src-tauri/src/database_pg/repositories/entity_name_repository.rs`
- Methoden: get_all, get_by_id, create, update, delete, count
- Entity-spezifische Methoden (z.B. get_by_booking)

### 3. Registrieren (repositories/mod.rs)
```rust
pub mod entity_name_repository;
pub use entity_name_repository::EntityNameRepository;
```

### 4. Commands hinzufügen (lib_pg.rs)
```rust
#[tauri::command]
async fn get_all_entities_pg(pool: State<'_, DbPool>) -> Result<Vec<EntityName>, String> {
    match EntityNameRepository::get_all(&pool).await {
        Ok(entities) => Ok(entities),
        Err(e) => Err(e.to_string())
    }
}
```

### 5. Registrieren (lib_pg.rs invoke_handler)
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing
    get_all_entities_pg,
    get_entity_by_id_pg,
    // ...
])
```

---

## 🎯 Erwartetes Ergebnis Session 3

### Nach Session 3:
- **9 Repositories fertig** (5 alt + 4 neu)
- **~1,685 Datensätze abgedeckt** (97%!)
- **~30 neue Commands**

### Fortschritts-Balken:
```
Repositories: █████████░░░░░░░░░░░░ 39% (9/23)
Commands:     █████████░░░░░░░░░░░░ 44% (~44/~100)
Datensätze:   ███████████████████░░ 97% (1,685/1,740)
```

---

## 💡 Pro-Tips

1. **Kopiere bestehende Repositories** - RoomRepository ist das beste Template
2. **Teste nach jedem Repository** - Cargo check nach jedem Schritt
3. **Dokumentiere während du arbeitest** - Update PROGRESS_SESSION_3.md
4. **Kleine Commits** - Nach jedem Repository committen
5. **Pausen machen** - Alle 2 Repositories kurze Pause

---

## 📂 Datei-Locations

```
src-tauri/src/
├── database_pg/
│   ├── models.rs                    ← Models hinzufügen
│   └── repositories/
│       ├── mod.rs                    ← Registrieren
│       ├── email_log_repository.rs   ← NEU
│       ├── reminder_repository.rs    ← NEU
│       ├── accompanying_guest_repository.rs  ← NEU
│       ├── service_template_repository.rs    ← NEU
│       └── discount_template_repository.rs   ← NEU
└── lib_pg.rs                         ← Commands hinzufügen
```

---

**Erstellt:** 2025-11-14 22:10
**Status:** ⏳ Ready to Start
**Viel Erfolg bei Session 3!** 🚀
