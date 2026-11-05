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

def wait_until_server_has_pages(count: int, timeout_secs: float = 5.0) -> None:
    import time
    deadline = time.time() + timeout_secs
    while time.time() < deadline:
        rows = query("SELECT page_id FROM pages")
        if len(rows) == count:
            return
        time.sleep(0.2)
    rows = query("SELECT page_id FROM pages")
    raise AssertionError(f"server pages count is {len(rows)}, wanted {count}")
