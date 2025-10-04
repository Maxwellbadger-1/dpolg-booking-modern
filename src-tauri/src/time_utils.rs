use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};

/// Timezone offset für UTC+2 (Deutschland Winterzeit / CEST Sommerzeit simuliert als festes UTC+2)
const TIMEZONE_OFFSET_HOURS: i32 = 2;

/// Gibt die aktuelle Zeit in UTC+2 zurück
pub fn now_utc_plus_2() -> DateTime<FixedOffset> {
    let offset = FixedOffset::east_opt(TIMEZONE_OFFSET_HOURS * 3600).unwrap();
    Utc::now().with_timezone(&offset)
}

/// Gibt das heutige Datum in UTC+2 als NaiveDate zurück
pub fn today_utc_plus_2() -> NaiveDate {
    now_utc_plus_2().date_naive()
}

/// Formatiert das aktuelle Datum in UTC+2 als String (DD.MM.YYYY)
pub fn format_today_de() -> String {
    now_utc_plus_2().format("%d.%m.%Y").to_string()
}

/// Formatiert die aktuelle Zeit in UTC+2 als String (DD.MM.YYYY HH:MM)
pub fn format_now_de() -> String {
    now_utc_plus_2().format("%d.%m.%Y %H:%M").to_string()
}

/// Formatiert das aktuelle Datum in UTC+2 als String (YYYY-MM-DD) für Datenbank
pub fn format_today_db() -> String {
    now_utc_plus_2().format("%Y-%m-%d").to_string()
}

/// Addiert Tage zum aktuellen Datum in UTC+2
pub fn add_days(days: i64) -> DateTime<FixedOffset> {
    now_utc_plus_2() + Duration::days(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_offset() {
        let now = now_utc_plus_2();
        // UTC+2 bedeutet 2 Stunden voraus
        assert_eq!(now.offset().local_minus_utc(), 2 * 3600);
    }

    #[test]
    fn test_format_functions() {
        let today = format_today_de();
        assert!(today.contains("."));

        let now = format_now_de();
        assert!(now.contains(":"));

        let db_date = format_today_db();
        assert!(db_date.contains("-"));
    }
}
