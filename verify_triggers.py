#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Verify that reminder NOTIFY triggers exist and are active"""

import psycopg2

DB_CONFIG = {
    'dbname': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'host': '141.147.3.123',
    'port': '5432',
    'connect_timeout': 10
}

try:
    conn = psycopg2.connect(**DB_CONFIG)
    cur = conn.cursor()

    print("=" * 80)
    print("VERIFYING REMINDER NOTIFY TRIGGERS")
    print("=" * 80)

    # Check if notify_table_change() function exists and handles reminders
    print("\n[1] Checking notify_table_change() function...")
    cur.execute("""
        SELECT pg_get_functiondef(p.oid)::text
        FROM pg_proc p
        JOIN pg_namespace n ON p.pronamespace = n.oid
        WHERE n.nspname = 'public'
        AND p.proname = 'notify_table_change';
    """)

    result = cur.fetchone()
    if result:
        func_def = result[0]
        if 'reminders' in func_def and 'reminder_changes' in func_def:
            print("   ✅ Function exists and handles 'reminders' table")
            print("   ✅ Function sends to 'reminder_changes' channel")
        else:
            print("   ❌ Function exists but does NOT handle 'reminders' table!")
            print("\n   Function definition:")
            print("   " + "\n   ".join(func_def.split('\n')[:30]))
    else:
        print("   ❌ Function notify_table_change() does NOT exist!")

    # Check reminder triggers
    print("\n[2] Checking reminder NOTIFY triggers...")
    cur.execute("""
        SELECT
            tgname AS trigger_name,
            tgrelid::regclass AS table_name,
            tgenabled AS enabled,
            CASE tgtype & 66
                WHEN 2 THEN 'BEFORE'
                WHEN 64 THEN 'INSTEAD OF'
                ELSE 'AFTER'
            END AS trigger_timing,
            CASE tgtype & 28
                WHEN 4 THEN 'INSERT'
                WHEN 8 THEN 'DELETE'
                WHEN 16 THEN 'UPDATE'
                WHEN 12 THEN 'INSERT OR DELETE'
                WHEN 20 THEN 'INSERT OR UPDATE'
                WHEN 24 THEN 'DELETE OR UPDATE'
                WHEN 28 THEN 'INSERT OR UPDATE OR DELETE'
            END AS trigger_event,
            pg_get_triggerdef(oid) as trigger_def
        FROM pg_trigger
        WHERE tgname LIKE 'trg_notify_reminder%'
        ORDER BY tgname;
    """)

    triggers = cur.fetchall()
    if triggers:
        print(f"   ✅ Found {len(triggers)} reminder NOTIFY triggers:")
        for trigger in triggers:
            name, table, enabled, timing, event, trigger_def = trigger
            status = "ENABLED" if enabled == 'O' else "DISABLED"
            print(f"\n   Trigger: {name}")
            print(f"      Table: {table}")
            print(f"      Status: {status}")
            print(f"      Timing: {timing} {event}")
            if enabled != 'O':
                print(f"      ⚠️ WARNING: Trigger is DISABLED!")
    else:
        print("   ❌ NO reminder NOTIFY triggers found!")
        print("   ⚠️ Migration 018 was NOT applied correctly!")

    # Check if triggers are firing (test with dummy update)
    print("\n[3] Testing if triggers actually fire...")
    print("   Creating test reminder...")

    # Get a booking ID for test
    cur.execute("SELECT id FROM bookings LIMIT 1")
    booking_id = cur.fetchone()[0]

    # Create test reminder
    cur.execute("""
        INSERT INTO reminders (booking_id, reminder_type, title, description, due_date, priority)
        VALUES ($1, 'test', 'Test Reminder', 'Test', CURRENT_DATE + INTERVAL '1 day', 'low')
        RETURNING id;
    """, [booking_id])

    test_reminder_id = cur.fetchone()[0]
    print(f"   ✅ Test reminder created: #{test_reminder_id}")

    # Listen for NOTIFY
    print("   Listening for NOTIFY events...")
    cur.execute("LISTEN reminder_changes;")
    conn.commit()

    # Update test reminder (should trigger NOTIFY)
    print("   Updating test reminder (should trigger NOTIFY)...")
    cur.execute("UPDATE reminders SET title = 'Test Updated' WHERE id = $1", [test_reminder_id])
    conn.commit()

    # Check for notifications
    conn.poll()
    if conn.notifies:
        notify = conn.notifies.pop(0)
        print(f"   ✅ NOTIFY received on channel: {notify.channel}")
        print(f"      Payload: {notify.payload}")
    else:
        print("   ❌ NO NOTIFY received!")
        print("   ⚠️ Triggers are NOT firing!")

    # Cleanup test reminder
    cur.execute("DELETE FROM reminders WHERE id = $1", [test_reminder_id])
    conn.commit()
    print(f"   🗑️ Test reminder deleted")

    print("\n" + "=" * 80)
    print("VERIFICATION COMPLETE")
    print("=" * 80)

    cur.close()
    conn.close()

except Exception as e:
    print(f"❌ Error: {e}")
    import traceback
    traceback.print_exc()
