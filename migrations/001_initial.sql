CREATE TABLE meters (
    id                  BIGSERIAL PRIMARY KEY,
    name                TEXT NOT NULL,
    unit                TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE meter_instances (
    id                  BIGSERIAL PRIMARY KEY,
    meter_id            BIGINT NOT NULL REFERENCES meters(id),
    meter_number        TEXT NOT NULL UNIQUE,
    initial_reading     NUMERIC(14, 3) NOT NULL,
    initial_reading_date DATE NOT NULL,
    installed_at        DATE NOT NULL,
    removed_at          DATE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT meter_instance_dates_valid
        CHECK (removed_at IS NULL OR removed_at >= installed_at)
);

CREATE TABLE readings (
    id                  BIGSERIAL PRIMARY KEY,
    meter_instance_id   BIGINT NOT NULL REFERENCES meter_instances(id),
    reading_date        DATE NOT NULL,
    value               NUMERIC(14, 3) NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (meter_instance_id, reading_date),
    CONSTRAINT reading_value_non_negative CHECK (value >= 0)
);

CREATE INDEX idx_readings_meter_instance_date
    ON readings (meter_instance_id, reading_date);

INSERT INTO meters (name, unit)
VALUES
    ('Electricity', 'kWh'),
    ('Water', 'm³'),
    ('Gas', 'm³');
