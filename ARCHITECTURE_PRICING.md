# 💰 Preisberechnung - Single Source of Truth Architecture

## 🎯 Ziel

**EINE zentrale Stelle** für ALLE Preisberechnungen. Alle anderen Komponenten konsumieren nur diese Daten.

## 🏗️ Architektur (Best Practice 2025)

```
┌─────────────────────────────────────────────────────────┐
│                   BACKEND (Rust)                        │
│  pricing.rs: calculate_booking_total()                  │
│  ↓ Single Source of Truth für ALLE Berechnungen        │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│              TAURI COMMAND (API Layer)                  │
│  calculate_booking_price_command()                      │
│  → Nimmt: BookingData (komplettes Objekt!)             │
│  → Gibt: FullPriceBreakdown (alle Details!)            │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│          FRONTEND (React Hook - Data Layer)             │
│  hooks/usePriceCalculation.ts                          │
│  → Cache für Performance                                │
│  → Event-basierte Auto-Updates                          │
│  → KEINE eigene Berechnung!                            │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│        UI KOMPONENTEN (Presentation Layer)              │
│  BookingSidebar, BookingDetails, etc.                   │
│  → Nutzen IMMER usePriceCalculation()                  │
│  → Zeigen nur Daten an, berechnen NIE selbst!         │
└─────────────────────────────────────────────────────────┘
```

## 📋 Problem: Aktuelle Architektur

### Aktuelle Situation (INKONSISTENT):

```typescript
// ❌ PROBLEM 1: Frontend berechnet selbst
const finalPrice = grundpreis * (service.original_value / 100);

// ❌ PROBLEM 2: Mehrere Backend-Calls
const basePriceResult = await invoke('calculate_booking_price_command', ...);
const priceResult = await invoke('calculate_booking_price_command', ...);

// ❌ PROBLEM 3: Duplikate Logik
// BookingSidebar.tsx Zeile 540-548
// BookingDetails.tsx Zeile 864
// ServiceTemplateList.tsx Zeile 146
// ... alle berechnen/formatieren selbst!
```

### Warum das Probleme verursacht:

1. **Chicken-Egg-Problem:** % Services brauchen Grundpreis, Grundpreis ändert sich durch Services
2. **Rundungsfehler:** Frontend berechnet mit float, Backend mit float → Differenzen
3. **Wartung unmöglich:** Änderung an Preislogik = 10+ Dateien anfassen
4. **Race Conditions:** Mehrere parallele Berechnungen → welche ist korrekt?

## ✅ Lösung: Neue Architektur

### Schritt 1: Backend - Vollständiges Preis-Objekt

```rust
// src-tauri/src/pricing.rs

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceCalculation {
    pub name: String,
    pub price_type: String,           // "fixed" | "percent"
    pub original_value: f64,          // Template-Wert (z.B. 19.0)
    pub applies_to: String,           // "overnight_price" | "total_price"
    pub calculated_price: f64,        // FINALER berechneter Preis
    pub base_amount: Option<f64>,     // Basis für % (z.B. Grundpreis)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullPriceBreakdown {
    // Basis
    pub base_price: f64,              // Grundpreis (Zimmer × Nächte)
    pub nights: i32,
    pub price_per_night: f64,
    pub is_hauptsaison: bool,

    // Services (MIT ALLEN Details!)
    pub services: Vec<ServiceCalculation>,
    pub services_total: f64,

    // Rabatte (MIT ALLEN Details!)
    pub discounts: Vec<DiscountCalculation>,
    pub discounts_total: f64,

    // Summen
    pub subtotal: f64,                // Grundpreis + Services
    pub total: f64,                   // Nach Rabatten

    // Meta
    pub calculation_timestamp: String,
}

// EINE zentrale Funktion für ALLE Berechnungen
pub fn calculate_full_booking_price(
    room_id: i64,
    checkin: &str,
    checkout: &str,
    is_member: bool,
    services: &[ServiceInput],        // Nimmt komplette Service-Objekte!
    discounts: &[DiscountInput],
    conn: &Connection,
) -> Result<FullPriceBreakdown, String> {

    // 1. Grundpreis berechnen
    let base_price = calculate_base_price(room_id, checkin, checkout, conn)?;

    // 2. Services berechnen (inkl. % vom Grundpreis)
    let mut calculated_services = Vec::new();
    let mut services_running_total = 0.0;

    for service in services {
        let calculated_price = match service.price_type.as_str() {
            "percent" => {
                let base = match service.applies_to.as_str() {
                    "overnight_price" => base_price,
                    "total_price" => base_price + services_running_total,
                    _ => base_price,
                };
                base * (service.original_value / 100.0)
            },
            "fixed" => service.original_value,
            _ => service.original_value,
        };

        services_running_total += calculated_price;

        calculated_services.push(ServiceCalculation {
            name: service.name.clone(),
            price_type: service.price_type.clone(),
            original_value: service.original_value,
            applies_to: service.applies_to.clone(),
            calculated_price,
            base_amount: Some(base_price),
        });
    }

    // 3. Rabatte berechnen
    let subtotal = base_price + services_running_total;
    let (calculated_discounts, discounts_total) =
        calculate_discounts(discounts, subtotal, base_price, is_member, conn)?;

    // 4. Finales Ergebnis
    Ok(FullPriceBreakdown {
        base_price,
        nights: calculate_nights(checkin, checkout)?,
        price_per_night: base_price / calculate_nights(checkin, checkout)? as f64,
        is_hauptsaison: is_hauptsaison(checkin, conn)?,
        services: calculated_services,
        services_total: services_running_total,
        discounts: calculated_discounts,
        discounts_total,
        subtotal,
        total: subtotal - discounts_total,
        calculation_timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
```

### Schritt 2: Tauri Command - Einfaches Interface

```rust
// src-tauri/src/lib.rs

#[derive(Deserialize)]
struct ServiceInput {
    name: String,
    price_type: String,
    original_value: f64,
    applies_to: String,
    template_id: Option<i64>,
}

#[tauri::command]
fn calculate_booking_price_command(
    room_id: i64,
    checkin: String,
    checkout: String,
    is_member: bool,
    services: Option<Vec<ServiceInput>>,  // Komplette Objekte!
    discounts: Option<Vec<DiscountInput>>,
) -> Result<FullPriceBreakdown, String> {
    let conn = Connection::open(database::get_db_path())
        .map_err(|e| format!("Datenbankfehler: {}", e))?;

    let services = services.unwrap_or_default();
    let discounts = discounts.unwrap_or_default();

    // EINE Funktion macht ALLES
    pricing::calculate_full_booking_price(
        room_id,
        &checkin,
        &checkout,
        is_member,
        &services,
        &discounts,
        &conn,
    )
}
```

### Schritt 3: Frontend Hook - Cache & Updates

```typescript
// src/hooks/usePriceCalculation.ts

interface PriceCalculationInput {
  roomId: number;
  checkin: string;
  checkout: string;
  isMember: boolean;
  services: ServiceInput[];
  discounts: DiscountInput[];
}

interface ServiceInput {
  name: string;
  priceType: 'fixed' | 'percent';
  originalValue: number;
  appliesTo: 'overnight_price' | 'total_price';
  templateId?: number;
}

export function usePriceCalculation(input: PriceCalculationInput | null) {
  const [priceBreakdown, setPriceBreakdown] = useState<FullPriceBreakdown | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Cache für Performance (verhindert unnötige Backend-Calls)
  const cacheKey = JSON.stringify(input);
  const cache = useRef<Map<string, FullPriceBreakdown>>(new Map());

  const calculate = useCallback(async () => {
    if (!input) {
      setPriceBreakdown(null);
      return;
    }

    // Cache-Check
    const cached = cache.current.get(cacheKey);
    if (cached) {
      setPriceBreakdown(cached);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // EINE Stelle für Backend-Call
      const result = await invoke<FullPriceBreakdown>(
        'calculate_booking_price_command',
        {
          roomId: input.roomId,
          checkin: input.checkin,
          checkout: input.checkout,
          isMember: input.isMember,
          services: input.services,
          discounts: input.discounts,
        }
      );

      // Cache speichern
      cache.current.set(cacheKey, result);
      setPriceBreakdown(result);
    } catch (err) {
      console.error('Preisberechnung fehlgeschlagen:', err);
      setError(String(err));
      setPriceBreakdown(null);
    } finally {
      setLoading(false);
    }
  }, [cacheKey, input]);

  // Auto-Berechnung bei Input-Änderung
  useEffect(() => {
    calculate();
  }, [calculate]);

  // Event-Listener für globale Updates
  useEffect(() => {
    const handlePriceUpdate = () => {
      cache.current.clear(); // Cache invalidieren
      calculate();
    };

    window.addEventListener('price-recalculation-needed', handlePriceUpdate);
    return () => window.removeEventListener('price-recalculation-needed', handlePriceUpdate);
  }, [calculate]);

  return {
    priceBreakdown,
    loading,
    error,
    recalculate: calculate,
  };
}
```

### Schritt 4: UI Komponenten - Nur Anzeige

```typescript
// src/components/BookingManagement/BookingSidebar.tsx

export default function BookingSidebar({ bookingId, isOpen, onClose }) {
  const [formData, setFormData] = useState<BookingFormData>({ ... });
  const [services, setServices] = useState<ServiceInput[]>([]);
  const [discounts, setDiscounts] = useState<DiscountInput[]>([]);

  // ✅ EINE Stelle für Preisberechnung
  const { priceBreakdown, loading, error } = usePriceCalculation({
    roomId: formData.room_id,
    checkin: formData.checkin_date,
    checkout: formData.checkout_date,
    isMember: guest?.dpolg_mitglied || false,
    services,
    discounts,
  });

  // ✅ UI zeigt nur Daten an - KEINE eigene Berechnung!
  return (
    <div>
      {loading && <Loader2 className="animate-spin" />}

      {priceBreakdown && (
        <div className="space-y-2">
          <div className="flex justify-between">
            <span>Grundpreis ({priceBreakdown.nights} Nächte)</span>
            <span>{priceBreakdown.basePrice.toFixed(2)} €</span>
          </div>

          {priceBreakdown.services.map((service, idx) => (
            <div key={idx} className="flex justify-between text-emerald-700">
              <span>
                + {service.name}
                {service.priceType === 'percent' && (
                  <span className="text-xs">
                    ({service.originalValue}% von {service.baseAmount?.toFixed(2)} €)
                  </span>
                )}
              </span>
              <span>{service.calculatedPrice.toFixed(2)} €</span>
            </div>
          ))}

          {priceBreakdown.discounts.map((discount, idx) => (
            <div key={idx} className="flex justify-between text-orange-700">
              <span>- {discount.name}</span>
              <span>{discount.calculatedAmount.toFixed(2)} €</span>
            </div>
          ))}

          <div className="flex justify-between font-bold pt-2 border-t">
            <span>Gesamtpreis:</span>
            <span>{priceBreakdown.total.toFixed(2)} €</span>
          </div>
        </div>
      )}
    </div>
  );
}
```

## 🎯 Vorteile der neuen Architektur

### 1. **Single Source of Truth**
```
✅ Backend berechnet ALLES
✅ Frontend zeigt nur an
✅ Keine Duplikate Logik
✅ EINE Änderung = überall konsistent
```

### 2. **Performance**
```
✅ Cache verhindert unnötige Calls
✅ EINE Berechnung statt 2-3
✅ Event-basierte Updates
```

### 3. **Wartbarkeit**
```
✅ Preislogik-Änderung = 1 Datei (pricing.rs)
✅ UI-Komponenten = pure presentation
✅ Tests einfach (1 Funktion testen)
```

### 4. **Debugging**
```
✅ Alle Berechnungen geloggt (Backend)
✅ Timestamp für jede Berechnung
✅ Kompletter Breakdown verfügbar
```

### 5. **Keine Rundungsfehler**
```
✅ Backend berechnet EINMAL mit korrekter Präzision
✅ Frontend nutzt fertige Werte
✅ Keine float-Arithmetik im Frontend
```

## 📊 Migrations-Plan

### Phase 1: Backend erweitern (1-2 Tage)
- [ ] `FullPriceBreakdown` Struct erstellen
- [ ] `calculate_full_booking_price()` implementieren
- [ ] `calculate_booking_price_command` refactoren
- [ ] Tests schreiben

### Phase 2: Frontend Hook (1 Tag)
- [ ] `usePriceCalculation.ts` erstellen
- [ ] Cache-Logik implementieren
- [ ] Event-System einbauen

### Phase 3: Komponenten migrieren (2-3 Tage)
- [ ] BookingSidebar refactoren
- [ ] BookingDetails refactoren
- [ ] ServiceTemplateList refactoren
- [ ] Alle anderen Komponenten

### Phase 4: Alte Logik entfernen (1 Tag)
- [ ] Alle manuellen Berechnungen löschen
- [ ] Utility-Functions nur für Formatierung behalten
- [ ] Code-Review & Testing

## 🔍 Vergleich: Vorher/Nachher

### VORHER (Aktuell):
```typescript
// ❌ 50+ Zeilen für Berechnung in BookingSidebar.tsx
const basePriceResult = await invoke('calculate_booking_price_command', ...);
const grundpreis = basePriceResult.grundpreis;
const calculatedServices = services.map(s => {
  let finalPrice = s.service_price;
  if (s.price_type === 'percent' && s.original_value && grundpreis > 0) {
    if (s.applies_to === 'overnight_price') {
      finalPrice = grundpreis * (s.original_value / 100);
    }
  }
  return [s.service_name, finalPrice];
});
const priceResult = await invoke('calculate_booking_price_command', ...);
// ... und dann NOCHMAL in BookingDetails.tsx
// ... und NOCHMAL in ServiceTemplateList.tsx
```

### NACHHER (Mit neuer Architektur):
```typescript
// ✅ 3 Zeilen - fertig!
const { priceBreakdown } = usePriceCalculation({
  roomId, checkin, checkout, isMember, services, discounts
});

// UI nutzt nur priceBreakdown.services[0].calculatedPrice
```

## 🚨 Wichtige Erkenntnisse

### 1. **Frontend sollte NIE berechnen**
```
Frontend = Presentation Layer
Backend = Business Logic Layer

Preisberechnung = Business Logic
→ Gehört ins Backend!
```

### 2. **"Chicken-Egg" ist kein Problem mit richtigem Design**
```
Backend macht EINE Berechnung:
1. Grundpreis
2. Services (% vom Grundpreis)
3. Subtotal (Grundpreis + Services)
4. Rabatte (% vom Subtotal)
5. Total

→ Alles in EINEM Durchlauf, keine Chicken-Egg!
```

### 3. **Caching ist kritisch**
```
Ohne Cache: Bei jeder onChange → Backend-Call
Mit Cache: Nur bei wirklicher Änderung

Performance-Gewinn: 10x-100x
```

## 📚 Referenzen

- **Clean Architecture** (Robert C. Martin) - Separation of Concerns
- **Domain-Driven Design** - Business Logic im Core
- **React Query Pattern** - Caching & State Management
- **Tauri Best Practices 2025** - Backend für Berechnungen nutzen

## ✅ Nächste Schritte

1. **Review diese Architektur** mit Team
2. **Entscheidung:** Jetzt refactoren oder nach Release?
3. **Bei "Ja":** Ich kann die Migration durchführen
4. **Bei "Später":** Dokumentation für zukünftige Refactoring

---

**Status:** 📋 Konzept-Phase
**Aufwand:** ~5-7 Tage für komplette Migration
**Gewinn:** Wartbarkeit ↑↑↑, Bugs ↓↓↓, Performance ↑
