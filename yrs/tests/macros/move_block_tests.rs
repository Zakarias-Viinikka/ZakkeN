macro_rules! make_page {
    ($user:expr, $time:expr, [$($content:expr),* $(,)?]) => {{
        let boss = Arc::new(BossOfYrs::new($user.to_string(), $time.to_string()));
        let mut block_ids: Vec<String> = Vec::new();
        $(
            block_ids.push(boss.clone().insert_new_block(
                $content.to_string(),
                String::new(),
                PositionToInsert::AtEnd,
            ).unwrap());
        )*
        (boss, block_ids)
    }};
}
