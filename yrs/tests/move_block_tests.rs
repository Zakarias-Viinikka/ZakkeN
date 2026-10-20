use std::sync::Arc;

use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::{BossOfYrs, doc_from_snapshot};

#[macro_use]
mod macros;

#[test]
fn concurrent_moves_of_same_block_land_it_once() {
    let (author, block_ids) = make_page!(
        "author",
        "1000",
        ["block 0", "block 1", "block 2", "block 3", "block 4"]
    );
    let snapshot = snapshot_boss(SnapshotBoss { boss: &author });

    let peer_a = new_peer_from(NewPeerFrom {
        snapshot: &snapshot,
        user: "peer_a",
        time: "2000",
    });
    let peer_b = new_peer_from(NewPeerFrom {
        snapshot: &snapshot,
        user: "peer_b",
        time: "3000",
    });

    move_block(MoveBlock {
        boss: &peer_a,
        block_id: &block_ids[4],
        block_id_of_above: Some(block_ids[0].as_str()),
        block_id_of_below: Some(block_ids[1].as_str()),
    });
    move_block(MoveBlock {
        boss: &peer_b,
        block_id: &block_ids[4],
        block_id_of_above: None,
        block_id_of_below: Some(block_ids[0].as_str()),
    });

    combine_yrs_pages(CombineYrsPages {
        page1: &peer_a,
        page2: &peer_b,
    });

    let blocks = read_all_blocks_ordered(ReadAllBlocksOrdered { boss: &peer_a });

    let position = blocks
        .iter()
        .position(|b| b.id_in_yrs == block_ids[4])
        .expect("block must be present");
    let result = position == 0 || position == 1;
    let expected_result = true;
    assert_eq!(result, expected_result);
}

#[test]
fn move_to_middle_and_back_restores_original_order() {
    let (boss, block_ids) = make_page!("u", "1000", ["0", "1", "2"]);

    move_block(MoveBlock {
        boss: &boss,
        block_id: &block_ids[2],
        block_id_of_above: Some(block_ids[0].as_str()),
        block_id_of_below: Some(block_ids[1].as_str()),
    });
    let result = read_block_ids(ReadBlockIds { boss: &boss });
    let expected_result = vec![block_ids[0].clone(), block_ids[2].clone(), block_ids[1].clone()];
    assert_eq!(result, expected_result);

    move_block(MoveBlock {
        boss: &boss,
        block_id: &block_ids[2],
        block_id_of_above: Some(block_ids[1].as_str()),
        block_id_of_below: None,
    });
    let result = read_block_ids(ReadBlockIds { boss: &boss });
    let expected_result = vec![block_ids[0].clone(), block_ids[1].clone(), block_ids[2].clone()];
    assert_eq!(result, expected_result);
}

#[test]
fn move_block_to_last_then_to_first() {
    let (boss, block_ids) = make_page!("u", "1000", ["0", "1", "2"]);

    move_block(MoveBlock {
        boss: &boss,
        block_id: &block_ids[0],
        block_id_of_above: Some(block_ids[2].as_str()),
        block_id_of_below: None,
    });
    let result = read_block_ids(ReadBlockIds { boss: &boss });
    let expected_result = vec![block_ids[1].clone(), block_ids[2].clone(), block_ids[0].clone()];
    assert_eq!(result, expected_result);

    move_block(MoveBlock {
        boss: &boss,
        block_id: &block_ids[0],
        block_id_of_above: None,
        block_id_of_below: Some(block_ids[1].as_str()),
    });
    let result = read_block_ids(ReadBlockIds { boss: &boss });
    let expected_result = vec![block_ids[0].clone(), block_ids[1].clone(), block_ids[2].clone()];
    assert_eq!(result, expected_result);
}

// ============================================================
// HELPERS
// ============================================================

struct SnapshotBoss<'a> {
    boss: &'a Arc<BossOfYrs>,
}

fn snapshot_boss(params: SnapshotBoss) -> Vec<u8> {
    params.boss.clone().snapshot().unwrap()
}

struct NewPeerFrom<'a> {
    snapshot: &'a [u8],
    user: &'a str,
    time: &'a str,
}

fn new_peer_from(params: NewPeerFrom) -> Arc<BossOfYrs> {
    Arc::new(
        doc_from_snapshot(
            params.snapshot.to_vec(),
            params.user.to_string(),
            "page".to_string(),
            params.time.to_string(),
        )
        .unwrap(),
    )
}

struct MoveBlock<'a> {
    boss: &'a Arc<BossOfYrs>,
    block_id: &'a str,
    block_id_of_above: Option<&'a str>,
    block_id_of_below: Option<&'a str>,
}

fn move_block(params: MoveBlock) {
    params
        .boss
        .clone()
        .move_block(
            params.block_id.to_string(),
            params.block_id_of_above.map(String::from),
            params.block_id_of_below.map(String::from),
        )
        .unwrap();
}

struct CombineYrsPages<'a> {
    page1: &'a Arc<BossOfYrs>,
    page2: &'a Arc<BossOfYrs>,
}

fn combine_yrs_pages(params: CombineYrsPages) {
    params
        .page1
        .clone()
        .merge_with(params.page2.clone())
        .unwrap();
}

struct ReadAllBlocksOrdered<'a> {
    boss: &'a Arc<BossOfYrs>,
}

fn read_all_blocks_ordered(params: ReadAllBlocksOrdered) -> Vec<my_yrs_lib::Block> {
    params.boss.clone().get_entire_page().unwrap()
}

struct ReadBlockIds<'a> {
    boss: &'a Arc<BossOfYrs>,
}

fn read_block_ids(params: ReadBlockIds) -> Vec<String> {
    read_all_blocks_ordered(ReadAllBlocksOrdered { boss: params.boss })
        .into_iter()
        .map(|b| b.id_in_yrs)
        .collect()
}