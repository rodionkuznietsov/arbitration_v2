CREATE SCHEMA IF NOT EXISTS storage;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'timeframe') THEN
        CREATE TYPE timeframe AS enum ('1m');
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'exchange_type') THEN
        CREATE TYPE exchange_type AS enum ('Bybit', 'Gate', 'KuCoin');
    END IF;
END$$;

CREATE TABLE IF NOT EXISTS storage.lines (
    timestamp BIGINT NOT NULL,
    long_exchange exchange_type NOT NULL,
    short_exchange exchange_type NOT NULL,
    symbol VARCHAR(255) NOT NULL,
    timeframe timeframe NOT NULL,
    value FLOAT NOT NULL,
    PRIMARY KEY (timestamp, long_exchange, short_exchange, symbol)
)