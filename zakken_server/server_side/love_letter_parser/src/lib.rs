use love_letter::LoveLetter;

pub fn get_target_page_id_from_letter(letter: &LoveLetter) -> String {
    match letter {
        LoveLetter::EditBlock { target_page_id, .. } => target_page_id.clone(),
        LoveLetter::CreateNewBlock { target_page_id, .. } => target_page_id.clone(),
        LoveLetter::CreateNewBlockWithId { target_page_id, .. } => target_page_id.clone(),
        LoveLetter::MoveBlock { target_page_id, .. } => target_page_id.clone(),
        LoveLetter::RemoveBlock { target_page_id, .. } => target_page_id.clone(),
        LoveLetter::DisablePage { page_id, .. } => page_id.clone(),
        LoveLetter::CreateNewPage { page_id, .. } => page_id.clone(),
    }
}