# Preisberechnung - Technische Dokumentation

**Version:** 1.9.0
**Stand:** 2025-02-12
**Zweck:** Detaillierte Erklärung der Preisberechnungslogik im DPolG Buchungssystem

---

## Überblick

Das Buchungssystem verwendet einen **Two-Pass Algorithmus** zur Preisberechnung mit automatischer Neuberechnung bei Änderungen preisrelevanter Felder. Alle Berechnungen erfolgen **im Backend (Rust)**, das Frontend zeigt nur gespeicherte Werte an.

---

## Single Source of Truth Prinzip

### Backend: Berechnungslogik
- **Funktion:** `calculate_full_booking_price_pg()` in `src-tauri/src/lib_pg.rs`
- **Verantwortung:** Alle Preisberechnungen durchführen
- **Trigger:** Erstellen, Updaten, manuelle Neuberechnung

### Frontend: Display-Formatierung
- **Funktion:** `calculateServicePrice()` in `src/utils/priceFormatting.ts`
- **Verantwortung:** Nur Formatierung bereits gespeicherter Werte für Anzeige
- **Verwendung:** Ausschließlich View-Mode (keine DB-Operationen)

**Wichtig:** Der Name `calculateServicePrice()` im Frontend ist historisch bedingt. Die Funktion führt KEINE echten Berechnungen durch, die in der DB gespeichert werden.

---

## Two-Pass Algorithmus

### Pass 1: Festbeträge + Prozentuale Services auf overnight_price

```rust
// Basis: overnight_price (Anzahl Nächte × Zimmerpreis pro Nacht)
let base_price = anzahl_naechte * preis_pro_nacht;

// Services berechnen:
for service in services {
    if service.price_type == "fixed" {
        first_pass_total += service.original_value;
    } else if service.applies_to == "overnight_price" {
        first_pass_total += base_price * (service.original_value / 100.0);
    }
}

// Endreinigung hinzufügen (wenn nicht bereits vorhanden)
if !has_endreinigung_service {
    first_pass_total += room.endreinigung;
}
```

### Pass 2: Prozentuale Services auf total_price

```rust
// Neue Basis: overnight_price + Pass 1 Services
let total_price_base = base_price + first_pass_total;

// Services berechnen:
for service in services {
    if service.price_type == "percent" && service.applies_to == "total_price" {
        second_pass_total += total_price_base * (service.original_value / 100.0);
    }
}
```

### Pass 3: Rabatte

```rust
// Basis abhängig von Einstellung "rabatt_basis"
let discount_base = match rabatt_basis {
    "overnight_price" => base_price,
    "total_price" => base_price + first_pass_total + second_pass_total,
};

for discount in discounts {
    if discount.discount_type == "fixed" {
        total_discounts += discount.amount;
    } else {
        total_discounts += discount_base * (discount.amount / 100.0);
    }
}
```

### Endpreis

```rust
final_price = base_price + first_pass_total + second_pass_total - total_discounts;
```

---

## Automatische Neuberechnung

### Trigger-Bedingungen

Bei Updates von Buchungen prüft das Backend automatisch ob eine Neuberechnung nötig ist:

```rust
let needs_price_recalc = checkin_date != old_booking.checkin_date
    || checkout_date != old_booking.checkout_date
    || room_id != old_booking.room_id
    || guest_id != old_booking.guest_id;
```

**Wichtig:** `anzahl_gaeste` ist **NICHT** preisrelevant!

### Warum ist Gästeanzahl nicht preisrelevant?

Im aktuellen System sind **Preise pro Zimmer**, nicht pro Person:
- Ein Zimmer kostet 100€/Nacht, unabhängig ob 1 oder 2 Personen darin schlafen
- Die `anzahl_gaeste` wird nur für Kapazitätsprüfung verwendet
- **Zukunft:** Wenn Per-Person-Preise eingeführt werden (z.B. `price_type="per_person"`), muss `anzahl_gaeste` zu den Trigger-Bedingungen hinzugefügt werden

### Was wird neu berechnet?

Bei automatischer Neuberechnung werden **alle prozentualen Services und Rabatte** mit den neuen Basiswerten neu berechnet:

1. **Datum-Änderung:** Neue Anzahl Nächte → neuer overnight_price
2. **Zimmer-Änderung:** Neuer Zimmerpreis + neue Endreinigung → neue Basis
3. **Gast-Änderung:** Neuer DPolG-Status → DPolG-Auto-Rabatt wird aktualisiert

**Festbeträge bleiben unverändert** (z.B. "Parkplatz 5€" bleibt 5€).

---

## Endreinigung Safeguards

### Problem: Doppelzählung vermeiden

Das System fügt automatisch `room.endreinigung` zur Preisberechnung hinzu. Aber was wenn der User manuell einen "Endreinigung" Service anlegt?

### Lösung: Safeguard in allen Berechnungsfunktionen

```rust
let has_endreinigung = services_list.iter()
    .any(|s| s.service_name.to_lowercase().contains("endreinigung")
          || s.service_name.to_lowercase().contains("cleaning"));

if !has_endreinigung {
    // Nur hinzufügen wenn NICHT bereits in Services
    first_pass_total += room.endreinigung;
}
```

**Implementiert in:**
- `calculate_full_booking_price_pg()` (Zeile 1951-1970)
- `recalculate_and_save_booking_prices()` (Zeile 2413-2424)
- `generate_invoice_html_pg()` (Invoice-Generierung)

---

## Services vs. Rabatte: Unterschiedliche Basis-Logik

### Services: Individuelles `applies_to` Feld

Jeder Service hat ein eigenes Feld:

```rust
pub struct Service {
    applies_to: String, // "overnight_price" oder "total_price"
}
```

**Beispiel:**
- Service 1: "Frühstück 10%" → applies_to = "overnight_price"
- Service 2: "Kurtaxe 5%" → applies_to = "total_price"

### Rabatte: Globales `rabatt_basis` Setting

Alle Rabatte teilen eine globale Einstellung:

```rust
// In settings table
rabatt_basis: "overnight_price" | "total_price"
```

**Beispiel:**
- `rabatt_basis = "overnight_price"` → ALLE Rabatte berechnen auf overnight_price
- `rabatt_basis = "total_price"` → ALLE Rabatte berechnen auf Gesamtpreis

### Warum dieser Unterschied?

**Design-Entscheidung:**
- **Services** sind unterschiedlich in ihrer Natur → individuelles `applies_to` sinnvoll
- **Rabatte** sollten konsistent berechnet werden → globales Setting verhindert Verwirrung

---

## View-Mode vs. Edit-Mode

### View-Mode (Historische Preise)

**Zweck:** Zeigt den Preis wie er zum Zeitpunkt der Buchung war

```typescript
// Frontend zeigt gespeicherte Werte
<div>{booking.calculated_price} €</div>

// Services zeigen gespeicherten calculated_price
<div>{service.calculated_price} €</div>
```

**Wichtig:** Verwendet `calculateServicePrice()` nur zur **Formatierung**, nicht zur Berechnung!

### Edit-Mode (Live-Berechnung)

**Zweck:** Zeigt den Preis mit aktuellen Zimmerpreisen

```typescript
// usePriceCalculation Hook
const { grundpreis, servicesTotal } = usePriceCalculation({
  checkin: new Date('2025-06-01'),
  checkout: new Date('2025-06-05'),
  room: selectedRoom,
  services: editedServices
});
```

**Backend berechnet beim Speichern:**
```rust
// Speichern triggert Neuberechnung
let calculated_price = calculate_full_booking_price_pg(&pool, booking_id).await?;
```

### Invoice (Rechnung)

**Prinzip:** Verwendet IMMER gespeicherte Preise aus der Buchung

```rust
// generate_invoice_html_pg()
// Liest booking.calculated_price, services[].calculated_price aus DB
// Keine Neuberechnung, zeigt historische Werte
```

**Warum?** Rechnungen müssen unveränderlich sein, auch wenn Zimmerpreise später steigen.

---

## Konsistenz-Garantien

### Sidebar = Invoice

**Problem (vor v1.9.0):** Sidebar zeigte andere Preise als Rechnung

**Lösung:**
1. **View-Mode:** Sidebar zeigt gespeicherte Preise (wie Rechnung)
2. **Edit-Mode:** Sidebar zeigt Live-Berechnung mit aktuellen Preisen
3. **Auto-Update:** Backend berechnet automatisch bei Änderungen neu

### Drei Orte, eine Logik

| Ort | Quelle | Zeitpunkt |
|-----|--------|-----------|
| **Sidebar (View)** | `booking.calculated_price` | Gespeichert |
| **Sidebar (Edit)** | `usePriceCalculation()` | Live |
| **Invoice** | `booking.calculated_price` | Gespeichert |

**Alle drei verwenden dieselbe Backend-Berechnung beim Speichern!**

---

## Kritische Funktionen

### Backend (Rust)

| Funktion | Datei | Zeile | Zweck |
|----------|-------|-------|-------|
| `calculate_full_booking_price_pg()` | lib_pg.rs | 1842-2055 | Hauptberechnung (Two-Pass) |
| `recalculate_and_save_booking_prices()` | lib_pg.rs | 2289-2550 | Neuberechnung + DB-Update |
| `update_booking_pg()` | lib_pg.rs | 619-640 | Auto-Trigger bei Updates |
| `generate_invoice_html_pg()` | lib_pg.rs | - | Invoice mit gespeicherten Preisen |

### Frontend (TypeScript)

| Funktion | Datei | Zweck |
|----------|-------|-------|
| `usePriceCalculation()` | hooks/usePriceCalculation.ts | Single Source of Truth Hook |
| `calculateServicePrice()` | utils/priceFormatting.ts | View-Mode Formatierung |
| `BookingSidebar` | components/BookingManagement/BookingSidebar.tsx | Preisanzeige |

---

## Häufige Szenarien

### Szenario 1: Buchung erstellen

1. User gibt Datum, Zimmer, Gast ein
2. Frontend ruft `usePriceCalculation()` für Live-Vorschau
3. User klickt "Speichern"
4. Backend ruft `calculate_full_booking_price_pg()`
5. Ergebnis wird in `bookings.calculated_price` gespeichert
6. Sidebar zeigt gespeicherten Wert

### Szenario 2: Datum ändern

1. User ändert `checkout_date` von 5.6. auf 7.6.
2. User klickt "Speichern"
3. Backend erkennt: `checkout_date != old_booking.checkout_date` → `needs_price_recalc = true`
4. Backend ruft `recalculate_and_save_booking_prices()`
5. Alle prozentualen Services/Rabatte werden mit neuer Basis neu berechnet
6. Neue Preise werden gespeichert

### Szenario 3: Zimmer ändern

1. User ändert Zimmer von "Zimmer 1" zu "Zimmer 5"
2. User klickt "Speichern"
3. Backend erkennt: `room_id != old_booking.room_id` → `needs_price_recalc = true`
4. Neue Basis: `zimmer5.nebensaison_preis * anzahl_naechte`
5. Neue Endreinigung: `zimmer5.endreinigung`
6. Alle prozentualen Services neu berechnet
7. Neue Preise gespeichert

### Szenario 4: Gast ändert DPolG-Status

1. User öffnet Gast-Details und ändert `is_dpolg_member = true`
2. User klickt "Speichern" (Gast)
3. User öffnet Buchung und ändert `guest_id` (oder lässt gleich)
4. Wenn `guest_id` ändert → `needs_price_recalc = true`
5. DPolG-Auto-Rabatt wird neu berechnet (falls aktiviert)

**Wichtig:** Änderung des DPolG-Status beim Gast triggert NICHT automatisch Neuberechnung aller Buchungen. Nur wenn die Buchung selbst bearbeitet wird (guest_id ändert), wird neu berechnet.

### Szenario 5: Service "Endreinigung" manuell hinzufügen

1. Zimmer hat `endreinigung = 50€`
2. User fügt manuell Service "Endreinigung 50€" hinzu
3. Backend prüft: `has_endreinigung = true` (Service-Name enthält "endreinigung")
4. Backend fügt Endreinigung NICHT automatisch hinzu
5. **Ergebnis:** Nur 50€ (kein Doppelzählung)

---

## Testing-Szenarien

### 1. Festbetrag-Service

```
Buchung: 3 Nächte × 100€ = 300€
Service: "Parkplatz 10€" (fixed)
Erwartung: 310€
```

### 2. Prozentuale Services (overnight_price)

```
Buchung: 3 Nächte × 100€ = 300€
Service 1: "Frühstück 10%" (overnight_price)
Erwartung: 300€ + 30€ = 330€
```

### 3. Prozentuale Services (total_price)

```
Buchung: 3 Nächte × 100€ = 300€
Service 1: "Frühstück 10€" (fixed) → 310€
Service 2: "Kurtaxe 5%" (total_price)
Erwartung: 310€ + 15.50€ = 325.50€
```

### 4. Rabatte (overnight_price Basis)

```
Buchung: 3 Nächte × 100€ = 300€
Rabatt: "DPolG 15%" (rabatt_basis = overnight_price)
Erwartung: 300€ - 45€ = 255€
```

### 5. Rabatte (total_price Basis)

```
Buchung: 3 Nächte × 100€ = 300€
Service: "Frühstück 20€" (fixed) → 320€
Rabatt: "DPolG 15%" (rabatt_basis = total_price)
Erwartung: 320€ - 48€ = 272€
```

### 6. Endreinigung Safeguard

```
Zimmer: endreinigung = 50€
Services: "Endreinigung 50€" (manuell hinzugefügt)
Erwartung: Nur 50€ (nicht 100€)
```

---

## Zukunftssichere Erweiterungen

### Per-Person Pricing

Wenn in Zukunft Per-Person-Preise implementiert werden:

1. **Neue Felder:**
   - `rooms.price_type` (ENUM: "per_room", "per_person")
   - `rooms.price_per_person` (DECIMAL)

2. **Trigger-Bedingung erweitern:**
   ```rust
   let needs_price_recalc = checkin_date != old_booking.checkin_date
       || checkout_date != old_booking.checkout_date
       || room_id != old_booking.room_id
       || guest_id != old_booking.guest_id
       || anzahl_gaeste != old_booking.anzahl_gaeste; // NEU!
   ```

3. **Basispreis-Berechnung:**
   ```rust
   let base_price = if room.price_type == "per_person" {
       anzahl_naechte * room.price_per_person * booking.anzahl_gaeste
   } else {
       anzahl_naechte * room.nebensaison_preis
   };
   ```

---

## Zusammenfassung

| Aspekt | Beschreibung |
|--------|--------------|
| **Algorithmus** | Two-Pass: Services (overnight/total), dann Rabatte |
| **Single Source** | Backend berechnet, Frontend zeigt an |
| **Auto-Update** | Bei Datum/Zimmer/Gast Änderung |
| **Gästeanzahl** | NICHT preisrelevant (Preise pro Zimmer) |
| **Endreinigung** | Safeguards gegen Doppelzählung |
| **Konsistenz** | View-Mode = Invoice (gespeicherte Preise) |
| **Services** | Individuelles `applies_to` |
| **Rabatte** | Globales `rabatt_basis` Setting |

**Status:** Produktionsreif, keine bekannten Bugs ✅
