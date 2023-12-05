CREATE TABLE IF NOT EXISTS messages(
    conversation_id TEXT,
    msec REAL,
    prompt TEXT,
    response TEXT
);

CREATE TABLE IF NOT EXISTS conversations(
    id TEXT,
    msec REAL
);