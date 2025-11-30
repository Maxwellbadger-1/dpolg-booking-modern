#!/usr/bin/env python3
"""
PostgreSQL Schema Validator
Vergleicht PostgreSQL Spalten mit Rust row.get() calls
"""

import re
from collections import defaultdict

def parse_pg_schema(file_path):
    """Parse PostgreSQL schema from exported file"""
    tables = defaultdict(set)

    with open(file_path, 'r') as f:
        for line in f:
            # Parse: table_name | column_name | data_type
            parts = line.strip().split('|')
            if len(parts) >= 2 and parts[0].strip() and parts[1].strip():
                table = parts[0].strip()
                column = parts[1].strip()

                # Skip header lines
                if table == 'table_name' or table.startswith('---'):
                    continue

                tables[table].add(column)

    return tables

def parse_rust_column_access(file_path):
    """Parse Rust row.get() calls"""
    columns_by_file = defaultdict(set)

    with open(file_path, 'r') as f:
        for line in f:
            # Format: file:line:code
            match = re.search(r'([^:]+):\d+:.*row\.get\("(\w+)"\)', line)
            if match:
                file_name = match.group(1)
                column = match.group(2)
                columns_by_file[file_name].add(column)

    return columns_by_file

def guess_table_from_filename(file_name):
    """Try to guess table name from repository file name"""
    # e.g., "booking_repository.rs" -> "bookings"
    file_base = file_name.split('/')[-1].replace('_repository.rs', '')

    # Simple pluralization
    if file_base.endswith('y'):
        return file_base[:-1] + 'ies'  # company -> companies
    elif file_base == 'accompanying_guest':
        return 'accompanying_guests'
    elif file_base == 'guest':
        return 'guests'
    elif not file_base.endswith('s'):
        return file_base + 's'
    else:
        return file_base

def find_mismatches(pg_tables, rust_columns):
    """Find schema mismatches"""
    mismatches = []

    for file_name, columns in rust_columns.items():
        # Skip non-repository files
        if '_repository.rs' not in file_name:
            continue

        table = guess_table_from_filename(file_name)

        if table not in pg_tables:
            # Try alternative names
            if table.endswith('s'):
                table_singular = table[:-1]
                if table_singular in pg_tables:
                    table = table_singular
                else:
                    mismatches.append(f"⚠️  Table guess failed for {file_name} (guessed: {table})")
                    continue
            else:
                mismatches.append(f"⚠️  Table guess failed for {file_name} (guessed: {table})")
                continue

        pg_cols = pg_tables[table]

        for col in columns:
            if col not in pg_cols:
                mismatches.append(f"❌ {table}: column '{col}' not in PostgreSQL schema (used in {file_name})")

    return mismatches

def main():
    print("🔍 PostgreSQL Schema Validator")
    print("=" * 60)

    # Parse files
    print("\n📊 Parsing PostgreSQL schema...")
    pg_tables = parse_pg_schema('docs/pg_schema_full.txt')
    print(f"   Found {len(pg_tables)} tables")

    print("\n🦀 Parsing Rust column access...")
    rust_columns = parse_rust_column_access('docs/rust_column_access.txt')
    print(f"   Found {len(rust_columns)} repository files")

    # Find mismatches
    print("\n🔎 Checking for mismatches...")
    mismatches = find_mismatches(pg_tables, rust_columns)

    print("\n" + "=" * 60)
    if mismatches:
        print(f"🔴 FOUND {len(mismatches)} SCHEMA MISMATCHES:\n")
        for m in mismatches:
            print(f"  {m}")
        print("\n" + "=" * 60)
        return 1
    else:
        print("✅ All schemas match! No mismatches found.")
        print("=" * 60)
        return 0

if __name__ == '__main__':
    exit(main())
