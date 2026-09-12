#!/usr/bin/env bash

set -euo pipefail

ENVIRONMENT="${1:-}"

case "$ENVIRONMENT" in
    test)
        ENV_FILE=".env.test"
        EXPECTED_DATABASE="metervalues_test"
        ;;
    dev)
        ENV_FILE=".env.dev"
        EXPECTED_DATABASE="metervalues_dev"
        ;;
    *)
        echo "Usage: $0 {test|dev}"
        exit 1
        ;;
esac

if [[ ! -f "$ENV_FILE" ]]; then
    echo "ERROR: $ENV_FILE not found"
    exit 1
fi

# Load database configuration.
set -a
source "$ENV_FILE"
set +a

# Safety check.
if [[ "${DATABASE_URL}" != *"$EXPECTED_DATABASE"* ]]; then
    echo "ERROR: DATABASE_URL does not point to $EXPECTED_DATABASE"
    exit 1
fi

echo "=== DATABASE: $EXPECTED_DATABASE ==="
echo

psql "$DATABASE_URL" <<'SQL'

\echo '=== METERS ==='
SELECT *
FROM meters
ORDER BY id;

\echo ''
\echo '=== METER INSTANCES ==='
SELECT *
FROM meter_instances
ORDER BY id;

\echo ''
\echo '=== READINGS ==='
SELECT *
FROM readings
ORDER BY meter_instance_id, reading_date, id;

SQL
