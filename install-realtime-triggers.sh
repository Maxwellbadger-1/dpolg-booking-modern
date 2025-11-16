#!/bin/bash
# ============================================================================
# Install PostgreSQL NOTIFY Triggers for Real-Time Collaboration
# ============================================================================
# Usage: ./install-realtime-triggers.sh
# Requirements: PostgreSQL connection configured in .env
# ============================================================================

set -e  # Exit on error

echo "🚀 Installing PostgreSQL NOTIFY Triggers for Real-Time Collaboration"
echo ""

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "❌ Error: .env file not found!"
    echo "   Please create .env with DATABASE_URL"
    exit 1
fi

# Parse DATABASE_URL
# Format: postgres://user:password@host:port/database
if [ -z "$DATABASE_URL" ]; then
    echo "❌ Error: DATABASE_URL not set in .env"
    exit 1
fi

# Extract connection details
DB_URL=$DATABASE_URL
echo "📡 Connecting to PostgreSQL..."
echo "   URL: ${DB_URL//:*@/:***@}"  # Hide password in logs
echo ""

# Install triggers
echo "⚙️  Installing NOTIFY triggers..."
cat src-tauri/database_notifications.sql | psql "$DB_URL"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ SUCCESS: Triggers installed!"
    echo ""
    echo "📊 Verifying installation..."
    psql "$DB_URL" -c "
        SELECT
            trigger_name,
            event_object_table AS table_name,
            action_timing || ' ' || event_manipulation AS event
        FROM information_schema.triggers
        WHERE trigger_name LIKE '%notify%'
        ORDER BY event_object_table, trigger_name;
    "
    echo ""
    echo "🎉 Real-Time Collaboration System is ready!"
    echo ""
    echo "Next steps:"
    echo "  1. Run Rust backend to start LISTEN"
    echo "  2. Open app in multiple browser tabs"
    echo "  3. Make changes → See live updates!"
else
    echo ""
    echo "❌ ERROR: Failed to install triggers"
    exit 1
fi
