#!/usr/bin/env bash

set -euo pipefail

# Load test database configuration.
set -a
source .env.test
set +a

# Safety check.
if [[ "${DATABASE_URL}" != *"metervalues_test"* ]]; then
    echo "ERROR: DATABASE_URL does not point to metervalues_test"
    exit 1
fi

echo "=== DATABASE: metervalues_test ==="
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
