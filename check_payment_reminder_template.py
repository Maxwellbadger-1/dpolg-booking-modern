#!/usr/bin/env python3
"""
Check if payment_reminder template exists
"""
import sys
import psycopg2

DB_CONFIG = {
    'host': '141.147.3.123',
    'database': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'port': 5432
}

try:
    conn = psycopg2.connect(**DB_CONFIG)
    cursor = conn.cursor()

    print("=== EMAIL TEMPLATES CHECK ===\n")

    # Check all templates
    cursor.execute("SELECT template_name, subject, is_active FROM email_templates ORDER BY template_name;")
    templates = cursor.fetchall()

    print(f"Found {len(templates)} email templates:")
    for name, subject, is_active in templates:
        status = "[ACTIVE]" if is_active else "[INACTIVE]"
        print(f"   {status} {name}: {subject[:60]}...")

    # Check specifically for payment_reminder
    cursor.execute("SELECT COUNT(*) FROM email_templates WHERE template_name = 'payment_reminder';")
    has_payment_reminder = cursor.fetchone()[0]

    print(f"\nPayment reminder template exists: {has_payment_reminder > 0}")

    cursor.close()
    conn.close()

except Exception as e:
    print(f"Error: {e}")
    sys.exit(1)
