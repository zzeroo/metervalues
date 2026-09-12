BEGIN;

-- ============================================================
-- METERS
-- ============================================================

INSERT INTO meters (name, unit)
VALUES
    ('Solar Production', 'kWh'),
    ('Heating Energy', 'kWh');


-- ============================================================
-- METER INSTANCES
-- ============================================================

-- Electricity: old meter replaced by current meter

INSERT INTO meter_instances (
    meter_id,
    meter_number,
    initial_reading,
    initial_reading_date,
    installed_at,
    removed_at
)
SELECT
    id,
    'ELEC-OLD-001',
    0.000,
    '2022-01-01',
    '2022-01-01',
    '2024-06-15'
FROM meters
WHERE name = 'Electricity';

INSERT INTO meter_instances (
    meter_id,
    meter_number,
    initial_reading,
    initial_reading_date,
    installed_at
)
SELECT
    id,
    'ELEC-NEW-002',
    15432.500,
    '2024-06-15',
    '2024-06-15'
FROM meters
WHERE name = 'Electricity';


-- Gas: one active meter

INSERT INTO meter_instances (
    meter_id,
    meter_number,
    initial_reading,
    initial_reading_date,
    installed_at
)
SELECT
    id,
    'GAS-001',
    0.000,
    '2023-01-01',
    '2023-01-01'
FROM meters
WHERE name = 'Gas';


-- Water: one active meter

INSERT INTO meter_instances (
    meter_id,
    meter_number,
    initial_reading,
    initial_reading_date,
    installed_at
)
SELECT
    id,
    'WATER-001',
    0.000,
    '2023-01-01',
    '2023-01-01'
FROM meters
WHERE name = 'Water';


-- Solar: active meter

INSERT INTO meter_instances (
    meter_id,
    meter_number,
    initial_reading,
    initial_reading_date,
    installed_at
)
SELECT
    id,
    'SOLAR-001',
    0.000,
    '2024-01-01',
    '2024-01-01'
FROM meters
WHERE name = 'Solar Production';


-- Heating: intentionally no meter instance
-- Useful for testing the empty state in the GUI.


-- ============================================================
-- READINGS
-- ============================================================

-- Old electricity meter

INSERT INTO readings (
    meter_instance_id,
    reading_date,
    value
)
SELECT
    mi.id,
    reading_date,
    value
FROM meter_instances mi
CROSS JOIN (
    VALUES
        ('2022-01-01'::date, 0.000::numeric),
        ('2022-06-01'::date, 2100.500::numeric),
        ('2023-01-01'::date, 4520.750::numeric),
        ('2023-06-01'::date, 6890.250::numeric),
        ('2024-01-01'::date, 9125.000::numeric),
        ('2024-06-15'::date, 11234.500::numeric)
) AS readings(reading_date, value)
WHERE mi.meter_number = 'ELEC-OLD-001';


-- Current electricity meter

INSERT INTO readings (
    meter_instance_id,
    reading_date,
    value
)
SELECT
    mi.id,
    reading_date,
    value
FROM meter_instances mi
CROSS JOIN (
    VALUES
        ('2024-06-15'::date, 15432.500::numeric),
        ('2024-09-01'::date, 16120.750::numeric),
        ('2025-01-01'::date, 17245.000::numeric),
        ('2025-06-01'::date, 18670.250::numeric),
        ('2026-01-01'::date, 20110.500::numeric),
        ('2026-08-01'::date, 21542.750::numeric)
) AS readings(reading_date, value)
WHERE mi.meter_number = 'ELEC-NEW-002';


-- Gas readings

INSERT INTO readings (
    meter_instance_id,
    reading_date,
    value
)
SELECT
    mi.id,
    reading_date,
    value
FROM meter_instances mi
CROSS JOIN (
    VALUES
        ('2023-01-01'::date, 0.000::numeric),
        ('2023-06-01'::date, 850.000::numeric),
        ('2024-01-01'::date, 1720.500::numeric),
        ('2025-01-01'::date, 2980.250::numeric),
        ('2026-01-01'::date, 4210.750::numeric),
        ('2026-08-01'::date, 4875.500::numeric)
) AS readings(reading_date, value)
WHERE mi.meter_number = 'GAS-001';


-- Water readings

INSERT INTO readings (
    meter_instance_id,
    reading_date,
    value
)
SELECT
    mi.id,
    reading_date,
    value
FROM meter_instances mi
CROSS JOIN (
    VALUES
        ('2023-01-01'::date, 0.000::numeric),
        ('2023-06-01'::date, 62.500::numeric),
        ('2024-01-01'::date, 135.000::numeric),
        ('2025-01-01'::date, 278.750::numeric),
        ('2026-01-01'::date, 421.250::numeric),
        ('2026-08-01'::date, 495.750::numeric)
) AS readings(reading_date, value)
WHERE mi.meter_number = 'WATER-001';


-- Solar readings

INSERT INTO readings (
    meter_instance_id,
    reading_date,
    value
)
SELECT
    mi.id,
    reading_date,
    value
FROM meter_instances mi
CROSS JOIN (
    VALUES
        ('2024-01-01'::date, 0.000::numeric),
        ('2024-06-01'::date, 1845.500::numeric),
        ('2025-01-01'::date, 3420.750::numeric),
        ('2025-06-01'::date, 5980.250::numeric),
        ('2026-01-01'::date, 8210.500::numeric),
        ('2026-08-01'::date, 10985.750::numeric)
) AS readings(reading_date, value)
WHERE mi.meter_number = 'SOLAR-001';


COMMIT;
