CREATE SCHEMA IF NOT EXISTS storage;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'timeframe') THEN
        CREATE TYPE timeframe AS enum ('1m');
    END IF;
END$$;

CREATE TABLE IF NOT EXISTS storage.lines (
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    exchange_pair VARCHAR(255) NOT NULL,
    symbol VARCHAR(255) NOT NULL,
    timeframe timeframe NOT NULL,
    value NUMERIC NOT NULL,
    PRIMARY KEY (timestamp, exchange_pair, symbol)
)