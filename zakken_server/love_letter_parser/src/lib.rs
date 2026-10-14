use love_letter::{BrokenHeart, LoveLetter, deserialize_love_letter};

pub fn get_target_page_id(letter_bytes: Vec<u8>) -> Result<String, BrokenHeart> {
    let letter = deserialize_love_letter(letter_bytes)?;
    Ok(match letter {
        LoveLetter::EditBlock { target_page_id, .. } => target_page_id,
        LoveLetter::CreateNewBlock { target_page_id, .. } => target_page_id,
        LoveLetter::RemoveBlock { target_page_id, .. } => target_page_id,
        LoveLetter::DisablePage { page_id, .. } => page_id,
        LoveLetter::CreateNewPage { page_id, .. } => page_id,
    })
}
