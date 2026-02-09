CREATE SCHEMA IF NOT EXISTS storage;

CREATE TABLE storage.candles (
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    exchange VARCHAR(255) NOT NULL,
    symbol VARCHAR(255) NOT NULL,
    interval VARCHAR(255) NOT NULL,
    open NUMERIC NOT NULL,
    high NUMERIC NOT NULL,
    low NUMERIC NOT NULL,
    close NUMERIC NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    PRIMARY KEY (timestamp, user_id, exchange, symbol)
)