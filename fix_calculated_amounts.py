#!/usr/bin/env python3
"""
Migration Script: Fix veraltete calculated_amount Werte in discounts Tabelle
Bug-Fix für: Preisaufschlüsselung zeigt veraltete Rabattbeträge nach Drag & Drop

Dieser Script repariert alle Buchungen, wo die calculated_amount in der discounts
Tabelle nicht mit den tatsächlichen Preisen übereinstimmt.
"""

import psycopg2
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Database connection
DB_HOST = os.getenv('DB_HOST', 'localhost')
DB_PORT = os.getenv('DB_PORT', '5432')
DB_NAME = os.getenv('DB_NAME', 'dpolg_booking')
DB_USER = os.getenv('DB_USER', 'postgres')
DB_PASSWORD = os.getenv('DB_PASSWORD', '')

def fix_calculated_amounts():
    """Fix veraltete calculated_amount Werte für alle Rabatte"""

    conn = psycopg2.connect(
        host=DB_HOST,
        port=DB_PORT,
        database=DB_NAME,
        user=DB_USER,
        password=DB_PASSWORD
    )

    cursor = conn.cursor()

    print("🔍 Checking for discounts with outdated calculated_amount values...")

    # Find all discounts with calculated_amount that doesn't match the expected value
    cursor.execute("""
        SELECT
            d.id,
            d.booking_id,
            d.discount_name,
            d.discount_type,
            d.discount_value,
            d.calculated_amount as old_calculated,
            b.grundpreis,
            b.services_preis,
            CASE
                WHEN d.discount_type = 'percent' THEN
                    (b.grundpreis + b.services_preis) * (d.discount_value / 100.0)
                ELSE
                    d.discount_value
            END as new_calculated
        FROM discounts d
        JOIN bookings b ON d.booking_id = b.id
        WHERE d.calculated_amount IS NOT NULL
          AND ABS(d.calculated_amount - CASE
              WHEN d.discount_type = 'percent' THEN
                  (b.grundpreis + b.services_preis) * (d.discount_value / 100.0)
              ELSE
                  d.discount_value
              END) > 0.01
        ORDER BY d.booking_id, d.id
    """)

    outdated_discounts = cursor.fetchall()

    if not outdated_discounts:
        print("✅ No outdated calculated_amount values found. Database is consistent!")
        cursor.close()
        conn.close()
        return

    print(f"⚠️  Found {len(outdated_discounts)} discounts with outdated calculated_amount values:\n")

    for row in outdated_discounts:
        discount_id, booking_id, name, dtype, value, old_calc, grundpreis, services, new_calc = row
        print(f"  Booking {booking_id}: '{name}'")
        print(f"    Type: {dtype}, Value: {value}")
        print(f"    Old calculated_amount: {old_calc:.2f} €")
        print(f"    New calculated_amount: {new_calc:.2f} € (based on grundpreis={grundpreis:.2f} + services={services:.2f})")
        print(f"    Difference: {abs(old_calc - new_calc):.2f} €\n")

    # Ask for confirmation
    confirm = input(f"Fix {len(outdated_discounts)} discounts? (yes/no): ").strip().lower()

    if confirm != 'yes':
        print("❌ Migration cancelled.")
        cursor.close()
        conn.close()
        return

    # Update all outdated discounts
    print("\n🔧 Updating calculated_amount values...")

    updated_count = 0
    for row in outdated_discounts:
        discount_id, booking_id, name, dtype, value, old_calc, grundpreis, services, new_calc = row

        cursor.execute("""
            UPDATE discounts
            SET calculated_amount = %s
            WHERE id = %s
        """, (new_calc, discount_id))

        updated_count += 1
        print(f"  ✓ Updated discount {discount_id} ('{name}' for booking {booking_id}): {old_calc:.2f} → {new_calc:.2f} €")

    # Commit changes
    conn.commit()

    print(f"\n✅ Migration completed successfully!")
    print(f"   Updated {updated_count} discount calculated_amount values")

    cursor.close()
    conn.close()

if __name__ == "__main__":
    print("=" * 80)
    print("Migration: Fix veraltete calculated_amount Werte")
    print("=" * 80)
    print()

    try:
        fix_calculated_amounts()
    except Exception as e:
        print(f"❌ Migration failed: {e}")
        import traceback
        traceback.print_exc()
