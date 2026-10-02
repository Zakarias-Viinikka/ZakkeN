use std::collections::HashMap;

use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
use leptos_use::storage::use_local_storage;

use crate::{db::ui_actions, shared_structs::LocalPages};

pub struct CheckboxToCallbackMap {
    pub map: HashMap<String, Box<dyn Fn() + Send + Sync>>,
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
    ) -> (Self, CheckboxToCallbackMap) {
        let mut map: HashMap<String, RwSignal<Checkbox>> = HashMap::new();
        let mut callback_map: HashMap<String, Box<dyn Fn() + Send + Sync>> = HashMap::new();

        add_to_both_maps(
            "insert_three_pages",
            &mut map,
            &mut callback_map,
            Box::new(move || insert_three_pages(ctr, local_pages_set)),
        );

        (
            Self {
                checkboxes: RwSignal::new(map),
            },
            CheckboxToCallbackMap { map: callback_map },
        )
    }
}

fn add_to_both_maps(
    key: &str,
    map: &mut HashMap<String, RwSignal<Checkbox>>,
    callback_map: &mut HashMap<String, Box<dyn Fn() + Send + Sync>>,
    callback: Box<dyn Fn() + Send + Sync>,
) {
    map.insert(key.into(), RwSignal::new(Checkbox::new(key)));
    callback_map.insert(key.into(), callback);
}

pub fn insert_three_pages(ctr: RwSignal<usize>, local_pages_set: WriteSignal<Vec<LocalPages>>) {
    ui_actions::create_new_page(ctr, local_pages_set);
    ui_actions::create_new_page(ctr, local_pages_set);
    ui_actions::create_new_page(ctr, local_pages_set);
}
