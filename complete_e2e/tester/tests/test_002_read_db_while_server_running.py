from support import db


def test_can_read_db_while_server_running(web_app, pause):
    tables = db.list_tables()

    web_app.evaluate(
        f"window.dispatchEvent(new CustomEvent('show_popup', {{ detail: 'read {len(tables)} tables from db' }}))"
    )
    pause(2)

    assert tables, f"expected at least one table in {db.get_path()}, got none"