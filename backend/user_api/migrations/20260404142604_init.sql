CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    tg_user_id BIGINT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS user_exchanges (
    id SERIAL PRIMARY KEY,
    tg_user_id BIGINT NOT NULL REFERENCES users(tg_user_id) ON DELETE CASCADE,
    exchange_name VARCHAR(255) NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    api_secret VARCHAR(255) NOT NULL
);

CREATE TABLE IF NOT EXISTS user_logs (
    tg_user_id BIGINT NOT NULL REFERENCES users(tg_user_id) ON DELETE CASCADE,
    event VARCHAR(255) NOT NULL,
    symbol VARCHAR(255) NOT NULL,
    long_exchange VARCHAR(255) NOT NULL,
    short_exchange VARCHAR(255) NOT NULL,
    timestamp BIGINT NOT NULL,
    PRIMARY KEY (timestamp, tg_user_id, event)
);

CREATE TABLE IF NOT EXISTS exchanges (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    is_available BOOLEAN NOT NULL DEFAULT TRUE
);