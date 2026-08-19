# Meter Values

Small Rust + PostgreSQL application for monthly Electricity, Water and Gas meter readings.

## Design

The important distinction is between:

- `meters` — logical meter type, e.g. Electricity / Water / Gas
- `meter_instances` — the physical meter currently/previously installed by the provider
- `readings` — readings belonging to one physical meter instance

This means a provider meter exchange does not break consumption calculations.

Each physical meter has:

- meter number
- initial reading
- initial reading date
- installation date
- optional removal date

The initial reading is the starting point for that physical meter. Subsequent readings are stored separately.

## Development database

Create `.env` from `.env.dev.example` and set the PostgreSQL credentials.

The application automatically applies pending SQLx migrations at startup.

Run:

```bash
cargo run
```

Default development address:

```text
http://127.0.0.1:8000
```

## Production database

Create `.env` from `.env.prod.example` and set the PostgreSQL credentials.

Build:

```bash
cargo build --release
```

Run:

```bash
./target/release/metervalues
```


## Test Database

Create `.env` from `.env.test.example` and set the PostgreSQL credentials.

Set environment variables from `.env.test` and run tests:

```bash
set -a
source .env.test
set +a
cargo test
```



## Important

The application intentionally does not contain Docker or a reverse proxy yet.

The first milestone is only:

- Rust application scaffold
- PostgreSQL connection
- SQLx migrations
- development/production configuration
- static frontend
- health endpoint
