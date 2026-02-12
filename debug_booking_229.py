#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Debug Script: Zeigt den aktuellen Status von Buchung 229
Prüft Preise, Rabatte und calculated_amount Werte
"""

import psycopg2
import os
import sys

# Ensure UTF-8 encoding for Windows console
if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8')

# Database connection (from pool.rs)
DB_HOST = '141.147.3.123'
DB_PORT = '5432'
DB_NAME = 'dpolg_booking'
DB_USER = 'dpolg_admin'
DB_PASSWORD = 'DPolG2025SecureBooking'

def debug_booking_229():
    conn = psycopg2.connect(
        host=DB_HOST,
        port=DB_PORT,
        database=DB_NAME,
        user=DB_USER,
        password=DB_PASSWORD
    )

    cursor = conn.cursor()

    print("=" * 80)
    print("DEBUG: Buchung 229 - Preisaufschlüsselung")
    print("=" * 80)
    print()

    # 1. Lade Buchung
    cursor.execute("""
        SELECT id, reservierungsnummer, checkin_date, checkout_date, anzahl_naechte,
               grundpreis, services_preis, rabatt_preis, gesamtpreis, guest_id, room_id
        FROM bookings
        WHERE id = 229
    """)

    booking = cursor.fetchone()
    if not booking:
        print("❌ Buchung 229 nicht gefunden!")
        return

    b_id, reserv_nr, checkin, checkout, naechte, grund, services, rabatt, gesamt, guest_id, room_id = booking

    print("📋 BUCHUNG:")
    print(f"  ID: {b_id}")
    print(f"  Reservierungsnummer: {reserv_nr}")
    print(f"  Zeitraum: {checkin} bis {checkout} ({naechte} Nächte)")
    print(f"  Guest ID: {guest_id}, Room ID: {room_id}")
    print()
    print("💰 PREISE IN BOOKINGS TABELLE:")
    print(f"  Grundpreis:     {grund:.2f} €")
    print(f"  Services:       {services:.2f} €")
    print(f"  Rabatt (Summe): -{rabatt:.2f} €")
    print(f"  Gesamtpreis:    {gesamt:.2f} €")
    print()
    print(f"  🧮 Rechnung: {grund:.2f} + {services:.2f} - {rabatt:.2f} = {grund + services - rabatt:.2f} €")
    print(f"  {'✅ STIMMT' if abs((grund + services - rabatt) - gesamt) < 0.01 else '❌ STIMMT NICHT'}")
    print()

    # 2. Lade Rabatte
    cursor.execute("""
        SELECT id, discount_name, discount_type, discount_value, calculated_amount
        FROM discounts
        WHERE booking_id = 229
        ORDER BY id
    """)

    discounts = cursor.fetchall()

    print("🎟️  RABATTE IN DISCOUNTS TABELLE:")
    if not discounts:
        print("  Keine Rabatte gefunden!")
    else:
        discount_sum = 0
        for d_id, name, dtype, value, calc_amt in discounts:
            print(f"  ID: {d_id}")
            print(f"    Name: '{name}'")
            print(f"    Type: {dtype}, Value: {value}")
            print(f"    calculated_amount: {calc_amt if calc_amt is not None else 'NULL'}")

            # Berechne erwarteten Wert
            if dtype == 'percent':
                expected = (grund + services) * (value / 100.0)
            else:
                expected = value

            if calc_amt is not None:
                print(f"    Erwartet: {expected:.2f} €")
                diff = abs(calc_amt - expected)
                if diff > 0.01:
                    print(f"    ⚠️  UNTERSCHIED: {diff:.2f} € (DB hat {calc_amt:.2f}, sollte {expected:.2f} sein)")
                else:
                    print(f"    ✅ KORREKT")
                discount_sum += calc_amt
            else:
                print(f"    ⚠️  calculated_amount ist NULL! Sollte {expected:.2f} sein")
                discount_sum += expected
            print()

        print(f"📊 SUMME ALLER RABATTE (aus calculated_amount): {discount_sum:.2f} €")
        print(f"📊 RABATT_PREIS in bookings Tabelle:           {rabatt:.2f} €")

        diff = abs(discount_sum - rabatt)
        if diff > 0.01:
            print(f"❌ INKONSISTENZ: {diff:.2f} € Unterschied!")
        else:
            print(f"✅ KONSISTENT")

    print()

    # 3. Lade Gast (DPolG-Status)
    cursor.execute("""
        SELECT id, vorname, nachname, dpolg_mitglied
        FROM guests
        WHERE id = %s
    """, (guest_id,))

    guest = cursor.fetchone()
    if guest:
        g_id, vorname, nachname, dpolg = guest
        print("👤 GAST:")
        print(f"  ID: {g_id}")
        print(f"  Name: {vorname} {nachname}")
        print(f"  DPolG Mitglied: {'✅ JA' if dpolg else '❌ NEIN'}")
        print()

    # 4. Lade Services
    cursor.execute("""
        SELECT service_name, price_type, original_value, calculated_price
        FROM additional_services
        WHERE booking_id = 229
    """)

    services_list = cursor.fetchall()
    print("🛎️  SERVICES:")
    if not services_list:
        print("  Keine Services gefunden!")
    else:
        for sname, ptype, oval, calc in services_list:
            print(f"  - {sname}: {ptype}, {oval} → {calc:.2f} €")

    print()
    print("=" * 80)

    cursor.close()
    conn.close()

if __name__ == "__main__":
    try:
        debug_booking_229()
    except Exception as e:
        print(f"❌ Fehler: {e}")
        import traceback
        traceback.print_exc()
