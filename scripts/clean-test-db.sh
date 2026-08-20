#!/usr/bin/env bash

set -euo pipefail

# Load the test environment.
set -a
source .env.test
set +a

# Safety check: never clean the development or production database.
if [[ "${DATABASE_URL}" != *"metervalues_test"* ]]; then
    echo "ERROR: DATABASE_URL does not point to metervalues_test"
    echo "DATABASE_URL=${DATABASE_URL}"
    exit 1
fi

echo "Cleaning test database..."

psql "$DATABASE_URL" <<'SQL'
BEGIN;

DELETE FROM readings;
DELETE FROM meter_instances;

-- Keep the default meters used by the application/tests.
DELETE FROM meters
WHERE name NOT IN ('Electricity', 'Water', 'Gas');

COMMIT;
SQL

echo "Test database cleaned successfully."
