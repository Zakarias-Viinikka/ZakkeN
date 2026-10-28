pub fn new_local_block_id() -> String {
    let mut s = String::with_capacity(32);
    for _ in 0..4 {
        let n = (js_sys::Math::random() * 4294967296.0) as u32;
        s.push_str(&format!("{:08x}", n));
    }
    s
}
