use super::freeze_internals::{FreezeDocInput, FrozenBlock};

pub(crate) fn destruct_input(input: FreezeDocInput) -> (String, String, String, Vec<FrozenBlock>) {
    (input.page_id, input.user_id, input.time, input.blocks)
}
