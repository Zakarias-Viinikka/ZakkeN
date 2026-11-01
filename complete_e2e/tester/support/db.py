import sqlite3


DB_PATH = "/tmp/zakken_e2e_world.sqlite"


def get_path() -> str:
    return DB_PATH


def connect_readonly() -> sqlite3.Connection:
    return sqlite3.connect(f"file:{DB_PATH}?mode=ro", uri=True)


def list_tables() -> list[str]:
    import os
    print(f"db path: {DB_PATH}")
    print(f"db size: {os.path.getsize(DB_PATH) if os.path.exists(DB_PATH) else 'missing'}")
    conn = connect_readonly()
    try:
        cursor = conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        )
        return [row[0] for row in cursor.fetchall()]
    finally:
        conn.close()


def query(sql: str, params: tuple = ()) -> list[tuple]:
    conn = connect_readonly()
    try:
        return conn.execute(sql, params).fetchall()
    finally:
        conn.close()
