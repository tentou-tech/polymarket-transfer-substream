CREATE TABLE IF NOT EXISTS transfers (
    id TEXT PRIMARY KEY,
    evt_tx_hash VARCHAR(66),
    evt_index INT,
    evt_block_time TIMESTAMP,
    evt_block_number BIGINT,
    from_addr VARCHAR(42),
    to_addr VARCHAR(42),
    operator VARCHAR(42),
    token_id VARCHAR(100),
    value NUMERIC,
    "index" INT
);
