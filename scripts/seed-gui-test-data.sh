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

if [[ ! -f "seed-gui-test-data.sql" ]]; then
    echo "ERROR: seed-gui-test-data.sql not found"
    exit 1
fi

echo "Preparing $EXPECTED_DATABASE..."
echo

# Start with a clean database while preserving the default
# Electricity, Water and Gas meters.
./scripts/clean-db.sh "$ENVIRONMENT"

echo
echo "Loading GUI test data into $EXPECTED_DATABASE..."
echo

# Load the selected environment.
set -a
source "$ENV_FILE"
set +a

# Safety check.
if [[ "${DATABASE_URL}" != *"$EXPECTED_DATABASE"* ]]; then
    echo "ERROR: DATABASE_URL does not point to $EXPECTED_DATABASE"
    echo "DATABASE_URL=${DATABASE_URL}"
    exit 1
fi

psql "$DATABASE_URL" -f seed-gui-test-data.sql

echo
echo "GUI test data loaded successfully into $EXPECTED_DATABASE."
