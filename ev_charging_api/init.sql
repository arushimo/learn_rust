CREATE TABLE IF NOT EXISTS charging_sessions (
    id SERIAL PRIMARY KEY,
    vehicle_model VARCHAR(50) NOT NULL,
    charged_kwh INTEGER NOT NULL,
    start_time TIMESTAMPTZ NOT NULL
);
