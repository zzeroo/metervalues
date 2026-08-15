CREATE UNIQUE INDEX idx_one_active_meter_per_meter
    ON meter_instances (meter_id)
    WHERE removed_at IS NULL;
