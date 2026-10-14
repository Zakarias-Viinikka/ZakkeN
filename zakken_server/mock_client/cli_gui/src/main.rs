use mock_client_core::{EditKind, MockClient};
use std::io::Write;
use tokio::io::AsyncBufReadExt;

const USER_NAME: &str = "mock_client";
const SERVER_URL: &str = "ws://127.0.0.1:3000/ws";
const WIDTH: usize = 62;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        clear_screen();
        eprintln!("  error: {e}");
        std::process::exit(1);
    }
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    let _ = std::io::stdout().flush();
}

fn rule() {
    println!("{}", "─".repeat(WIDTH));
}

fn print_header(client: &MockClient) {
    rule();
    println!("  mock client");
    println!("  {} @ {}", client.user_name(), SERVER_URL);
    println!(
        "  {} page(s), {} active",
        client.page_count(),
        client.active_page_count()
    );
    rule();
    println!();
    println!("  pages");
    println!("    newpage        create a new page");
    println!("    disable        disable a random page");
    println!();
    println!("  blocks");
    println!("    edit           edit a random block (random edit type)");
    println!("    insert         edit a random block (force insert)");
    println!("    delete         edit a random block (force delete)");
    println!("    replace        edit a random block (force replace)");
    println!("    newblock       add an empty block to a random page");
    println!("    delblock       remove a random block from a random page");
    println!();
    println!("  other");
    println!("    pages          list loaded pages");
    println!("    quit           leave");
    rule();
    println!();
}

fn print_result(lines: &[String]) {
    for line in lines {
        println!("  {}", line);
    }
}

async fn run() -> Result<(), String> {
    let mut client = MockClient::connect(SERVER_URL, USER_NAME).await?;

    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);
    let mut line = String::new();

    // What to show under the header after the next redraw. Empty on first draw.
    let mut pending: Vec<String> = Vec::new();

    loop {
        clear_screen();
        print_header(&client);
        if !pending.is_empty() {
            print_result(&pending);
            println!();
            pending.clear();
        }

        print!("> ");
        let _ = std::io::stdout().flush();
        line.clear();
        let n = reader
            .read_line(&mut line)
            .await
            .map_err(|e| format!("stdin: {e}"))?;
        if n == 0 {
            println!();
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if matches!(trimmed, "quit" | "exit") {
            clear_screen();
            println!();
            println!("  bye");
            println!();
            break;
        }

        pending = match trimmed {
            "pages" => {
                let mut lines = Vec::new();
                let pages = client.pages_info();
                if pages.is_empty() {
                    lines.push("no pages".to_string());
                } else {
                    for p in pages {
                        let tag = if p.disabled { "  [disabled]" } else { "" };
                        lines.push(format!("{}{}", p.page_id, tag));
                        for b in &p.blocks {
                            let text = if b.text.is_empty() {
                                "(empty)".to_string()
                            } else {
                                b.text.clone()
                            };
                            lines.push(format!("    [{}] {}  {}", b.index, b.block_id, text));
                        }
                    }
                }
                lines
            }

            "newpage" => match client.create_page().await {
                Ok(id) => vec![
                    format!("created page {}", id),
                    format!(
                        "now {} page(s), {} active",
                        client.page_count(),
                        client.active_page_count()
                    ),
                ],
                Err(e) => vec![format!("couldn't create a page: {e}")],
            },

            "disable" => match client.disable_random_page().await {
                Ok(id) => vec![
                    format!("disabled page {}", id),
                    format!(
                        "now {} page(s), {} active",
                        client.page_count(),
                        client.active_page_count()
                    ),
                ],
                Err(e) => vec![format!("couldn't disable a page: {e}")],
            },

            "newblock" => match client.insert_block_on_random_page().await {
                Ok((page_id, block_id)) => vec![
                    format!("on page {}", page_id),
                    format!("added block {}", block_id),
                ],
                Err(e) => vec![format!("couldn't add a block: {e}")],
            },

            "delblock" => match client.delete_block_on_random_page().await {
                Ok((page_id, position, text)) => {
                    let shown = if text.is_empty() {
                        "(empty)".to_string()
                    } else {
                        format!("\"{}\"", text)
                    };
                    vec![
                        format!("on page {}", page_id),
                        format!("removed block at position {} ({})", position, shown),
                    ]
                }
                Err(e) => vec![format!("couldn't remove a block: {e}")],
            },

            "edit" => do_edit(&mut client, EditKind::Random).await,
            "insert" => do_edit(&mut client, EditKind::Insert).await,
            "delete" => do_edit(&mut client, EditKind::Delete).await,
            "replace" => do_edit(&mut client, EditKind::Replace).await,

            other => vec![
                format!("unknown command: '{}'", other),
                "the command list is above".to_string(),
            ],
        };
    }

    Ok(())
}

async fn do_edit(client: &mut MockClient, kind: EditKind) -> Vec<String> {
    match client.edit_random_page(kind).await {
        Ok((page_id, desc)) => vec![format!("on page {}", page_id), desc],
        Err(e) => vec![format!("edit failed: {e}")],
    }
}