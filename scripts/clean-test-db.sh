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

# Load the selected environment.
set -a
source "$ENV_FILE"
set +a

# Safety check: only allow the expected database.
if [[ "${DATABASE_URL}" != *"$EXPECTED_DATABASE"* ]]; then
    echo "ERROR: DATABASE_URL does not point to $EXPECTED_DATABASE"
    echo "DATABASE_URL=${DATABASE_URL}"
    exit 1
fi

echo "Cleaning database: $EXPECTED_DATABASE..."

psql "$DATABASE_URL" <<'SQL'
BEGIN;

DELETE FROM readings;
DELETE FROM meter_instances;

-- Keep the default meters used by the application/tests.
DELETE FROM meters
WHERE name NOT IN ('Electricity', 'Water', 'Gas');

COMMIT;
SQL

echo "Database $EXPECTED_DATABASE cleaned successfully."
