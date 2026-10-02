use std::{collections::HashMap, pin::Pin, sync::Arc};

use leptos::{prelude::*, reactive::spawn_local, server::codee::string::JsonSerdeCodec};
use leptos_use::storage::use_local_storage;

use crate::{db::ui_actions, shared_structs::LocalPages};

pub type AsyncCallback = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>>>;

pub struct CheckboxToCallbackMap {
    pub map: HashMap<String, AsyncCallback>,
}

#[derive(Clone)]
pub struct AllCheckboxes {
    pub checkboxes: RwSignal<HashMap<String, RwSignal<Checkbox>>>,
}

#[derive(Clone)]
pub struct Checkbox {
    pub key: String,
    pub is_active: Signal<bool>,
    pub is_active_set: WriteSignal<bool>,
    //pub callback: Box<dyn Fn() + Send + Sync>,
}

impl Checkbox {
    fn new(key: &str) -> Self {
        let (is_active, is_active_set, _) = use_local_storage::<bool, JsonSerdeCodec>(key);
        Self {
            key: key.into(),
            is_active,
            is_active_set,
        }
    }
}

impl AllCheckboxes {
    pub fn new(
        ctr: RwSignal<usize>,
        local_pages_set: WriteSignal<Vec<LocalPages>>,
    ) -> (Self, Arc<CheckboxToCallbackMap>) {
        let mut map: HashMap<String, RwSignal<Checkbox>> = HashMap::new();
        let mut callback_map: HashMap<String, AsyncCallback> = HashMap::new();

        add_to_both_maps(
            "insert_three_pages",
            &mut map,
            &mut callback_map,
            move || insert_three_pages(ctr, local_pages_set),
        );

        (
            Self {
                checkboxes: RwSignal::new(map),
            },
            Arc::new(CheckboxToCallbackMap { map: callback_map }),
        )
    }
}

fn add_to_both_maps<F, Fut>(
    key: &str,
    map: &mut HashMap<String, RwSignal<Checkbox>>,
    callback_map: &mut HashMap<String, AsyncCallback>,
    callback: F,
) where
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = ()> + 'static,
{
    let boxed: AsyncCallback = Box::new(move || Box::pin(callback()));
    map.insert(key.into(), RwSignal::new(Checkbox::new(key)));
    callback_map.insert(key.into(), boxed);
}

pub async fn insert_three_pages(
    ctr: RwSignal<usize>,
    local_pages_set: WriteSignal<Vec<LocalPages>>,
) {
    let _ = ui_actions::create_new_page(ctr, local_pages_set).await;
    let _ = ui_actions::create_new_page(ctr, local_pages_set).await;
    let _ = ui_actions::create_new_page(ctr, local_pages_set).await;
}
