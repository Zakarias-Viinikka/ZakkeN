use crate::broken_heart::BrokenHeart;
use crate::letter::{LoveLetter, LoveLetterResponse};
use crate::sketch::LoveLetterSketch;

#[uniffi::export]
pub fn sketch_to_bytes(sketch: &LoveLetterSketch) -> Result<Vec<u8>, BrokenHeart> {
    bincode::serialize(sketch).map_err(Into::into)
}

#[uniffi::export]
pub fn love_letter_to_bytes(letter: &LoveLetter) -> Result<Vec<u8>, BrokenHeart> {
    bincode::serialize(letter).map_err(Into::into)
}

#[uniffi::export]
pub fn deserialize_sketch(bytes: Vec<u8>) -> Result<LoveLetterSketch, BrokenHeart> {
    bincode::deserialize(&bytes).map_err(Into::into)
}

#[uniffi::export]
pub fn deserialize_love_letter(bytes: Vec<u8>) -> Result<LoveLetter, BrokenHeart> {
    bincode::deserialize(&bytes).map_err(Into::into)
}

#[uniffi::export]
pub fn love_letter_response_to_bytes(
    response: &LoveLetterResponse,
) -> Result<Vec<u8>, BrokenHeart> {
    bincode::serialize(response).map_err(Into::into)
}

#[uniffi::export]
pub fn deserialize_love_letter_response(
    bytes: Vec<u8>,
) -> Result<LoveLetterResponse, BrokenHeart> {
    bincode::deserialize(&bytes).map_err(Into::into)
}
