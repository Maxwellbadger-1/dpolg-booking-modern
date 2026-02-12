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
            print("   [OK] Function exists and handles 'reminders' table")
            print("   [OK] Function sends to 'reminder_changes' channel")
        else:
            print("   [ERROR] Function exists but does NOT handle 'reminders' table!")
            print("\n   Function definition snippet:")
            lines = func_def.split('\n')
            for i, line in enumerate(lines[10:25], start=10):
                print(f"   {i}: {line}")
    else:
        print("   [ERROR] Function notify_table_change() does NOT exist!")

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
            END AS trigger_event
        FROM pg_trigger
        WHERE tgname LIKE 'trg_notify_reminder%'
        ORDER BY tgname;
    """)

    triggers = cur.fetchall()
    if triggers:
        print(f"   [OK] Found {len(triggers)} reminder NOTIFY triggers:")
        for trigger in triggers:
            name, table, enabled, timing, event = trigger
            status = "ENABLED" if enabled == 'O' else "DISABLED"
            print(f"\n   - {name}")
            print(f"     Table: {table}")
            print(f"     Status: {status}")
            print(f"     Type: {timing} {event}")
            if enabled != 'O':
                print(f"     [WARNING] Trigger is DISABLED!")
    else:
        print("   [ERROR] NO reminder NOTIFY triggers found!")
        print("   [WARNING] Migration 018 was NOT applied correctly!")

    print("\n" + "=" * 80)
    print("VERIFICATION COMPLETE")
    print("=" * 80)

    cur.close()
    conn.close()

except Exception as e:
    print(f"[ERROR] {e}")
    import traceback
    traceback.print_exc()
