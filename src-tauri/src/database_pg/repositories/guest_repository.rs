use crate::database_pg::{DbPool, DbResult, Guest};

pub struct GuestRepository;

impl GuestRepository {
    /// Get all guests from the database
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<Guest>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, vorname, nachname, email, telefon, dpolg_mitglied,
                        strasse, plz, ort, mitgliedsnummer, notizen, beruf,
                        bundesland, dienststelle, created_at, anrede, geschlecht,
                        land, telefon_geschaeftlich, telefon_privat, telefon_mobil,
                        fax, geburtsdatum, geburtsort, sprache, nationalitaet,
                        identifikationsnummer, debitorenkonto, kennzeichen,
                        rechnungs_email, marketing_einwilligung, leitweg_id,
                        kostenstelle, tags, automail, automail_sprache
                 FROM guests
                 ORDER BY nachname, vorname",
                &[],
            )
            .await?;

        Ok(rows.into_iter().map(Guest::from).collect())
    }

    /// Get a guest by ID
    pub async fn get_by_id(pool: &DbPool, id: i32) -> DbResult<Guest> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "SELECT id, vorname, nachname, email, telefon, dpolg_mitglied,
                        strasse, plz, ort, mitgliedsnummer, notizen, beruf,
                        bundesland, dienststelle, created_at, anrede, geschlecht,
                        land, telefon_geschaeftlich, telefon_privat, telefon_mobil,
                        fax, geburtsdatum, geburtsort, sprache, nationalitaet,
                        identifikationsnummer, debitorenkonto, kennzeichen,
                        rechnungs_email, marketing_einwilligung, leitweg_id,
                        kostenstelle, tags, automail, automail_sprache
                 FROM guests
                 WHERE id = $1",
                &[&id],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Guest with ID {} not found", id)))?;

        Ok(Guest::from(row))
    }

    /// Create a new guest
    pub async fn create(
        pool: &DbPool,
        vorname: String,
        nachname: String,
        email: String,
        telefon: Option<String>,
        dpolg_mitglied: bool,
        strasse: Option<String>,
        plz: Option<String>,
        ort: Option<String>,
        mitgliedsnummer: Option<String>,
        notizen: Option<String>,
        beruf: Option<String>,
        bundesland: Option<String>,
        dienststelle: Option<String>,
        // Extended fields
        anrede: Option<String>,
        geschlecht: Option<String>,
        land: Option<String>,
        telefon_geschaeftlich: Option<String>,
        telefon_privat: Option<String>,
        telefon_mobil: Option<String>,
        fax: Option<String>,
        geburtsdatum: Option<String>,
        geburtsort: Option<String>,
        sprache: Option<String>,
        nationalitaet: Option<String>,
        identifikationsnummer: Option<String>,
        debitorenkonto: Option<String>,
        kennzeichen: Option<String>,
        rechnungs_email: Option<String>,
        marketing_einwilligung: Option<bool>,
        leitweg_id: Option<String>,
        kostenstelle: Option<String>,
        tags: Option<String>,
        automail: Option<bool>,
        automail_sprache: Option<String>,
    ) -> DbResult<Guest> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "INSERT INTO guests (
                    vorname, nachname, email, telefon, dpolg_mitglied,
                    strasse, plz, ort, mitgliedsnummer, notizen, beruf,
                    bundesland, dienststelle, anrede, geschlecht, land,
                    telefon_geschaeftlich, telefon_privat, telefon_mobil,
                    fax, geburtsdatum, geburtsort, sprache, nationalitaet,
                    identifikationsnummer, debitorenkonto, kennzeichen,
                    rechnungs_email, marketing_einwilligung, leitweg_id,
                    kostenstelle, tags, automail, automail_sprache, created_at
                 ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
                    $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
                    $31, $32, $33, $34, CURRENT_TIMESTAMP
                 ) RETURNING id, vorname, nachname, email, telefon, dpolg_mitglied,
                             strasse, plz, ort, mitgliedsnummer, notizen, beruf,
                             bundesland, dienststelle, created_at, anrede, geschlecht,
                             land, telefon_geschaeftlich, telefon_privat, telefon_mobil,
                             fax, geburtsdatum, geburtsort, sprache, nationalitaet,
                             identifikationsnummer, debitorenkonto, kennzeichen,
                             rechnungs_email, marketing_einwilligung, leitweg_id,
                             kostenstelle, tags, automail, automail_sprache",
                &[
                    &vorname, &nachname, &email, &telefon, &dpolg_mitglied,
                    &strasse, &plz, &ort, &mitgliedsnummer, &notizen, &beruf,
                    &bundesland, &dienststelle, &anrede, &geschlecht, &land,
                    &telefon_geschaeftlich, &telefon_privat, &telefon_mobil,
                    &fax, &geburtsdatum, &geburtsort, &sprache, &nationalitaet,
                    &identifikationsnummer, &debitorenkonto, &kennzeichen,
                    &rechnungs_email, &marketing_einwilligung, &leitweg_id,
                    &kostenstelle, &tags, &automail, &automail_sprache,
                ],
            )
            .await?;

        Ok(Guest::from(row))
    }

    /// Update an existing guest
    pub async fn update(
        pool: &DbPool,
        id: i32,
        vorname: String,
        nachname: String,
        email: String,
        telefon: Option<String>,
        dpolg_mitglied: bool,
        strasse: Option<String>,
        plz: Option<String>,
        ort: Option<String>,
        mitgliedsnummer: Option<String>,
        notizen: Option<String>,
        beruf: Option<String>,
        bundesland: Option<String>,
        dienststelle: Option<String>,
        // Extended fields
        anrede: Option<String>,
        geschlecht: Option<String>,
        land: Option<String>,
        telefon_geschaeftlich: Option<String>,
        telefon_privat: Option<String>,
        telefon_mobil: Option<String>,
        fax: Option<String>,
        geburtsdatum: Option<String>,
        geburtsort: Option<String>,
        sprache: Option<String>,
        nationalitaet: Option<String>,
        identifikationsnummer: Option<String>,
        debitorenkonto: Option<String>,
        kennzeichen: Option<String>,
        rechnungs_email: Option<String>,
        marketing_einwilligung: Option<bool>,
        leitweg_id: Option<String>,
        kostenstelle: Option<String>,
        tags: Option<String>,
        automail: Option<bool>,
        automail_sprache: Option<String>,
    ) -> DbResult<Guest> {
        let client = pool.get().await?;

        let row = client
            .query_one(
                "UPDATE guests SET
                    vorname = $2, nachname = $3, email = $4, telefon = $5,
                    dpolg_mitglied = $6, strasse = $7, plz = $8, ort = $9,
                    mitgliedsnummer = $10, notizen = $11, beruf = $12,
                    bundesland = $13, dienststelle = $14, anrede = $15,
                    geschlecht = $16, land = $17, telefon_geschaeftlich = $18,
                    telefon_privat = $19, telefon_mobil = $20, fax = $21,
                    geburtsdatum = $22, geburtsort = $23, sprache = $24,
                    nationalitaet = $25, identifikationsnummer = $26,
                    debitorenkonto = $27, kennzeichen = $28, rechnungs_email = $29,
                    marketing_einwilligung = $30, leitweg_id = $31,
                    kostenstelle = $32, tags = $33, automail = $34,
                    automail_sprache = $35
                 WHERE id = $1
                 RETURNING id, vorname, nachname, email, telefon, dpolg_mitglied,
                           strasse, plz, ort, mitgliedsnummer, notizen, beruf,
                           bundesland, dienststelle, created_at, anrede, geschlecht,
                           land, telefon_geschaeftlich, telefon_privat, telefon_mobil,
                           fax, geburtsdatum, geburtsort, sprache, nationalitaet,
                           identifikationsnummer, debitorenkonto, kennzeichen,
                           rechnungs_email, marketing_einwilligung, leitweg_id,
                           kostenstelle, tags, automail, automail_sprache",
                &[
                    &id, &vorname, &nachname, &email, &telefon, &dpolg_mitglied,
                    &strasse, &plz, &ort, &mitgliedsnummer, &notizen, &beruf,
                    &bundesland, &dienststelle, &anrede, &geschlecht, &land,
                    &telefon_geschaeftlich, &telefon_privat, &telefon_mobil,
                    &fax, &geburtsdatum, &geburtsort, &sprache, &nationalitaet,
                    &identifikationsnummer, &debitorenkonto, &kennzeichen,
                    &rechnungs_email, &marketing_einwilligung, &leitweg_id,
                    &kostenstelle, &tags, &automail, &automail_sprache,
                ],
            )
            .await
            .map_err(|_| crate::database_pg::DbError::NotFound(format!("Guest with ID {} not found", id)))?;

        Ok(Guest::from(row))
    }

    /// Delete a guest
    pub async fn delete(pool: &DbPool, id: i32) -> DbResult<()> {
        let client = pool.get().await?;

        let rows_affected = client
            .execute("DELETE FROM guests WHERE id = $1", &[&id])
            .await?;

        if rows_affected == 0 {
            return Err(crate::database_pg::DbError::NotFound(format!(
                "Guest with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Search guests by name or email
    pub async fn search(pool: &DbPool, query: String) -> DbResult<Vec<Guest>> {
        let client = pool.get().await?;

        let search_pattern = format!("%{}%", query.to_lowercase());

        let rows = client
            .query(
                "SELECT id, vorname, nachname, email, telefon, dpolg_mitglied,
                        strasse, plz, ort, mitgliedsnummer, notizen, beruf,
                        bundesland, dienststelle, created_at, anrede, geschlecht,
                        land, telefon_geschaeftlich, telefon_privat, telefon_mobil,
                        fax, geburtsdatum, geburtsort, sprache, nationalitaet,
                        identifikationsnummer, debitorenkonto, kennzeichen,
                        rechnungs_email, marketing_einwilligung, leitweg_id,
                        kostenstelle, tags, automail, automail_sprache
                 FROM guests
                 WHERE LOWER(vorname) LIKE $1
                    OR LOWER(nachname) LIKE $1
                    OR LOWER(email) LIKE $1
                 ORDER BY nachname, vorname",
                &[&search_pattern],
            )
            .await?;

        Ok(rows.into_iter().map(Guest::from).collect())
    }

    /// Get guests by DPolG membership status
    pub async fn get_by_membership(pool: &DbPool, is_member: bool) -> DbResult<Vec<Guest>> {
        let client = pool.get().await?;

        let rows = client
            .query(
                "SELECT id, vorname, nachname, email, telefon, dpolg_mitglied,
                        strasse, plz, ort, mitgliedsnummer, notizen, beruf,
                        bundesland, dienststelle, created_at, anrede, geschlecht,
                        land, telefon_geschaeftlich, telefon_privat, telefon_mobil,
                        fax, geburtsdatum, geburtsort, sprache, nationalitaet,
                        identifikationsnummer, debitorenkonto, kennzeichen,
                        rechnungs_email, marketing_einwilligung, leitweg_id,
                        kostenstelle, tags, automail, automail_sprache
                 FROM guests
                 WHERE dpolg_mitglied = $1
                 ORDER BY nachname, vorname",
                &[&is_member],
            )
            .await?;

        Ok(rows.into_iter().map(Guest::from).collect())
    }

    /// Get guest count
    pub async fn count(pool: &DbPool) -> DbResult<i64> {
        let client = pool.get().await?;

        let row = client
            .query_one("SELECT COUNT(*) as count FROM guests", &[])
            .await?;

        Ok(row.get("count"))
    }
}
