use crate::random_edit::{self, EditKind};
use crate::wire::{
    WsRead, WsWrite, decode_result, expect_response, send_get_all_pages,
};
use futures_util::StreamExt;
use love_letter::{LoveLetterSketch, build_a_love_letter, love_letter_to_bytes};
use my_yrs_lib::yrs_active_pages::YrsActivePages;
use my_yrs_lib::{BossOfYrs, doc_from_snapshot, yrs_wrapper::PositionToInsert};
use protocol::serialization::Convert;
use rand::RngExt;
use server_protocol::{
    GetAllYrsPagesPayload, LoginPayload, RegisterPayload, Request, Response, SendLoveLetterPayload,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio_tungstenite::connect_async;

const FAKE_TIME: &str = "";

pub struct MockClient {
    write: WsWrite,
    read: WsRead,
    next_msg_id: usize,
    user_name: String,
    pages: HashMap<String, Arc<BossOfYrs>>,
    page_status: HashMap<String, Vec<u8>>,
    disabled: HashSet<String>,
}

pub struct BlockInfo {
    pub index: u32,
    pub block_id: String,
    pub text: String,
}

pub struct PageInfo {
    pub page_id: String,
    pub disabled: bool,
    pub blocks: Vec<BlockInfo>,
}

impl MockClient {
    pub async fn connect(url: &str, user_name: &str) -> Result<Self, String> {
        let (ws, _) = connect_async(url)
            .await
            .map_err(|e| format!("could not connect to {url}: {e}"))?;
        let (write, read) = ws.split();

        let mut client = MockClient {
            write,
            read,
            next_msg_id: 1,
            user_name: user_name.to_string(),
            pages: HashMap::new(),
            page_status: HashMap::new(),
            disabled: HashSet::new(),
        };

        client.login_or_register().await?;
        client.load_pages().await?;
        Ok(client)
    }

    pub fn user_name(&self) -> &str {
        &self.user_name
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn active_page_count(&self) -> usize {
        self.pages
            .keys()
            .filter(|id| !self.disabled.contains(*id))
            .count()
    }

    pub fn pages_info(&self) -> Vec<PageInfo> {
        let mut out: Vec<PageInfo> = self
            .pages
            .iter()
            .map(|(id, boss)| PageInfo {
                page_id: id.clone(),
                disabled: self.disabled.contains(id),
                blocks: Arc::clone(boss)
                    .get_entire_page()
                    .unwrap_or_default()
                    .into_iter()
                    .enumerate()
                    .map(|(i, b)| BlockInfo {
                        index: i as u32,
                        block_id: b.id_in_yrs,
                        text: b.text,
                    })
                    .collect(),
            })
            .collect();
        out.sort_by(|a, b| a.page_id.cmp(&b.page_id));
        out
    }

    async fn send_raw(&mut self, request: Request, payload: Vec<u8>) -> Result<Response, String> {
        let id = self.next_msg_id;
        self.next_msg_id += 1;
        crate::wire::send_request(&mut self.write, id, request, payload)
            .await
            .map_err(|e| e.to_string())?;
        expect_response(&mut self.read, id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn send_unit_letter(&mut self, letter_bytes: Vec<u8>) -> Result<(), String> {
        let payload = SendLoveLetterPayload {
            love_letter: letter_bytes,
        };
        let resp = self
            .send_raw(Request::SendLoveLetter, payload.to_payload())
            .await?;
        decode_result::<()>(&resp.payload.0)
    }

    fn build_letter(
        &self,
        sketch: LoveLetterSketch,
        boss: &Arc<BossOfYrs>,
    ) -> Result<Vec<u8>, String> {
        let snapshot = Arc::clone(boss)
            .snapshot()
            .map_err(|e| format!("snapshot: {e}"))?;
        let letter = build_a_love_letter(sketch, snapshot);
        love_letter_to_bytes(&letter).map_err(|e| format!("serialize letter: {e}"))
    }

    async fn try_login(&mut self) -> Result<bool, String> {
        let payload = LoginPayload {
            user_name: self.user_name.clone(),
        };
        let resp = self
            .send_raw(Request::Login, payload.to_payload())
            .await?;
        match decode_result::<()>(&resp.payload.0) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn try_register(&mut self) -> Result<(), String> {
        let payload = RegisterPayload {
            user_name: self.user_name.clone(),
        };
        let resp = self
            .send_raw(Request::Register, payload.to_payload())
            .await?;
        decode_result::<()>(&resp.payload.0)
    }

    async fn login_or_register(&mut self) -> Result<(), String> {
        if self.try_login().await? {
            return Ok(());
        }

        if let Err(e) = self.try_register().await {
            return Err(format!("register failed for '{}': {e}", self.user_name));
        }

        if !self.try_login().await? {
            return Err(format!(
                "login after register failed for '{}'",
                self.user_name
            ));
        }

        Ok(())
    }

    async fn load_pages(&mut self) -> Result<(), String> {
        let id = self.next_msg_id;
        self.next_msg_id += 1;
        send_get_all_pages(&mut self.write, id)
            .await
            .map_err(|e| e.to_string())?;
        let resp = expect_response(&mut self.read, id)
            .await
            .map_err(|e| e.to_string())?;

        let payload = decode_result::<GetAllYrsPagesPayload>(&resp.payload.0)?;

        let unix_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("clock: {e}"))?
            .as_secs()
            .to_string();

        self.pages.clear();
        self.page_status.clear();
        self.disabled.clear();

        for p in payload.pages {
            let boss = doc_from_snapshot(
                p.blobbed_page,
                self.user_name.clone(),
                p.page_id.clone(),
                unix_time.clone(),
            )
            .map_err(|e| format!("doc_from_snapshot {}: {e}", p.page_id))?;

            if page_is_disabled(&p.page_status, &p.page_id) {
                self.disabled.insert(p.page_id.clone());
            }

            self.page_status.insert(p.page_id.clone(), p.page_status);
            self.pages.insert(p.page_id, Arc::new(boss));
        }
        Ok(())
    }

    fn pick_random_active_page(&self) -> Result<(String, Arc<BossOfYrs>), String> {
        let active: Vec<(String, Arc<BossOfYrs>)> = self
            .pages
            .iter()
            .filter(|(id, _)| !self.disabled.contains(*id))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if active.is_empty() {
            if self.pages.is_empty() {
                return Err("no pages".to_string());
            }
            return Err("all pages are disabled".to_string());
        }

        let mut rng = rand::rng();
        let idx = rng.random_range(0..active.len());
        Ok(active[idx].clone())
    }

    pub async fn create_page(&mut self) -> Result<String, String> {
        let boss = Arc::new(BossOfYrs::new(
            self.user_name.clone(),
            FAKE_TIME.to_string(),
        ));

        BossOfYrs::insert_new_block(
            Arc::clone(&boss),
            "".to_string(),
            "".to_string(),
            PositionToInsert::AtEnd,
        )
        .map_err(|e| format!("insert title block: {e}"))?;

        BossOfYrs::insert_new_block(
            Arc::clone(&boss),
            "".to_string(),
            "".to_string(),
            PositionToInsert::AtEnd,
        )
        .map_err(|e| format!("insert normal block: {e}"))?;

        let page_id = boss.page_id.clone();
        let sketch = LoveLetterSketch::CreateNewPage {
            page_id: page_id.clone(),
            is_main_menu_page: false,
        };
        let bytes = self.build_letter(sketch, &boss)?;
        self.send_unit_letter(bytes).await?;

        let fresh_status = Arc::new(YrsActivePages::new_empty())
            .snapshot()
            .map_err(|e| format!("page_status: {e}"))?;
        self.page_status.insert(page_id.clone(), fresh_status);
        self.pages.insert(page_id.clone(), boss);
        Ok(page_id)
    }

    pub async fn edit_random_page(&mut self, kind: EditKind) -> Result<(String, String), String> {
        let (page_id, boss) = self.pick_random_active_page()?;

        let outcome = random_edit::perform_edit(&boss, kind)?;
        let bytes = self.build_letter(outcome.sketch, &boss)?;
        self.send_unit_letter(bytes).await?;

        Ok((page_id, outcome.description))
    }

    pub async fn insert_block_on_random_page(&mut self) -> Result<(String, String), String> {
        let (page_id, boss) = self.pick_random_active_page()?;

        let new_block_id = BossOfYrs::insert_new_block(
            Arc::clone(&boss),
            "".to_string(),
            "".to_string(),
            PositionToInsert::AtEnd,
        )
        .map_err(|e| format!("insert_new_block: {e}"))?;

        let sketch = LoveLetterSketch::CreateNewBlock {
            position_to_insert: PositionToInsert::AtEnd,
            target_page_id: page_id.clone(),
        };
        let bytes = self.build_letter(sketch, &boss)?;
        self.send_unit_letter(bytes).await?;

        Ok((page_id, new_block_id))
    }

    pub async fn delete_block_on_random_page(&mut self) -> Result<(String, u32, String), String> {
        let (page_id, boss) = self.pick_random_active_page()?;

        let blocks: Vec<BlockInfo> = Arc::clone(&boss)
            .get_entire_page()
            .map_err(|e| format!("get_entire_page: {e}"))?
            .into_iter()
            .enumerate()
            .map(|(i, b)| BlockInfo {
                index: i as u32,
                block_id: b.id_in_yrs,
                text: b.text,
            })
            .collect();

        if blocks.is_empty() {
            return Err("page has no blocks".to_string());
        }

        let mut rng = rand::rng();
        let idx = rng.random_range(0..blocks.len());
        let victim = &blocks[idx];
        let position = victim.index;
        let block_id = victim.block_id.clone();
        let text = victim.text.clone();

        BossOfYrs::delete_block(Arc::clone(&boss), block_id)
            .map_err(|e| format!("delete_block: {e}"))?;

        let sketch = LoveLetterSketch::RemoveBlock {
            position,
            target_page_id: page_id.clone(),
        };
        let bytes = self.build_letter(sketch, &boss)?;
        self.send_unit_letter(bytes).await?;

        Ok((page_id, position, text))
    }

    pub async fn disable_random_page(&mut self) -> Result<String, String> {
        let (page_id, _) = self.pick_random_active_page()?;

        let stored = self
            .page_status
            .get(&page_id)
            .cloned()
            .ok_or_else(|| format!("no cached page_status for {page_id}"))?;

        let active = Arc::new(
            YrsActivePages::new(stored).map_err(|e| format!("YrsActivePages::new: {e}"))?,
        );
        YrsActivePages::mark_page_deleted(Arc::clone(&active), page_id.clone())
            .map_err(|e| format!("mark_page_deleted: {e}"))?;
        let new_status = YrsActivePages::snapshot(Arc::clone(&active))
            .map_err(|e| format!("page_status snapshot: {e}"))?;

        let sketch = LoveLetterSketch::DisablePage {
            page_id: page_id.clone(),
        };
        let letter = build_a_love_letter(sketch, new_status.clone());
        let bytes = love_letter_to_bytes(&letter).map_err(|e| format!("serialize letter: {e}"))?;
        self.send_unit_letter(bytes).await?;

        self.page_status.insert(page_id.clone(), new_status);
        self.disabled.insert(page_id.clone());
        Ok(page_id)
    }
}

fn page_is_disabled(status_blob: &[u8], page_id: &str) -> bool {
    if status_blob.is_empty() {
        return false;
    }
    match YrsActivePages::new(status_blob.to_vec()) {
        Ok(active) => !Arc::new(active)
            .is_page_active(page_id.to_string())
            .unwrap_or(true),
        Err(_) => false,
    }
}