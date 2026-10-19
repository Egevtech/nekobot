CREATE TABLE IF NOT EXISTS users(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    userid BIGINT UNIQUE,
    username CHAR(32),
    user_first_name CHAR(100),
    use_count INTEGER
)