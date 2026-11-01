//! Caller pov: the pure math behind dragging a row to a new position.
//! No DOM, no signals, no leptos — just numbers in, numbers out. The
//! drag module (which reads DOM rects and listens to mouse events) uses
//! these; the component owner also uses `apply_row_move_event` to
//! actually rewrite the visual order.
//!
//! Three pieces:
//!
//! 1. `insertion_slot_for_dragged_center`: given the current top-to-
//!    bottom order of the other rows (each with its vertical midpoint
//!    already measured in scroll-body content coordinates) and the
//!    dragged row's midpoint, return the insertion slot. The slot is
//!    the number of other rows whose midpoint sits above the dragged
//!    row's midpoint. The dragged row's own midpoint decides — not the
//!    pointer position — because the pointer started inside the row
//!    and can be off-centre. This matches cqrs's `locate_drop`.
//!
//! 2. `build_row_move_event`: given the current order, the moved row id,
//!    and a slot, build a RowMoveEvent naming the neighbours in the new
//!    order. Returns None when the slot equals the row's current
//!    position (a no-op drop) so no event is emitted for a drag that
//!    returns to where it started.
//!
//! 3. `apply_row_move_event`: take the current order and a
//!    RowMoveEvent, remove the moved row, and insert it after the
//!    above_row_id (or at index 0 when above_row_id is None). Returns
//!    the new order.
//!
//! The `below_row_id` on RowMoveEvent isn't used by
//! `apply_row_move_event` — it exists because cqrs's MoveEvent carries
//! both neighbours, and dropping it would make the two shapes diverge
//! for no reason. Callers that only have one neighbour (top or bottom)
//! set the other to None.

/// Same shape as cqrs's MoveEvent, minus the page ids (this viewer has
/// no pages). above_row_id is None when the row lands at the very top;
/// below_row_id is None when it lands at the very bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowMoveEvent {
    pub moved_row_id: usize,
    pub above_row_id: Option<usize>,
    pub below_row_id: Option<usize>,
}

/// How many other rows have their vertical midpoint above the dragged
/// row's midpoint. Each entry is (row_id, midpoint_y) for one of the
/// other rows, in the current top-to-bottom order. The row ids are kept
/// here so the caller can pass the same list to build_row_move_event;
/// the count itself only cares about the midpoints.
pub fn insertion_slot_for_dragged_center(
    other_row_midpoints_in_order: &[(usize, f32)],
    dragged_center_y: f32,
) -> usize {
    other_row_midpoints_in_order
        .iter()
        .filter(|(_, mid)| *mid < dragged_center_y)
        .count()
}

/// Build a RowMoveEvent for moving `moved_row_id` to `slot` in the
/// order-with-the-moved-row-removed. Returns None when the slot equals
/// the moved row's current position (dropping it where it started is a
/// no-op and should not emit an event).
pub fn build_row_move_event(
    order: &[usize],
    moved_row_id: usize,
    slot: usize,
) -> Option<RowMoveEvent> {
    let current_position = order.iter().position(|&id| id == moved_row_id)?;
    let others: Vec<usize> = order
        .iter()
        .copied()
        .filter(|&id| id != moved_row_id)
        .collect();
    if slot > others.len() {
        return None;
    }
    let current_slot = order[..current_position]
        .iter()
        .filter(|&&id| id != moved_row_id)
        .count();
    if slot == current_slot {
        return None;
    }
    let above_row_id = if slot == 0 {
        None
    } else {
        Some(others[slot - 1])
    };
    let below_row_id = if slot == others.len() {
        None
    } else {
        Some(others[slot])
    };
    Some(RowMoveEvent {
        moved_row_id,
        above_row_id,
        below_row_id,
    })
}

/// Apply a RowMoveEvent to the order. Removes the moved row, then
/// inserts it after above_row_id (or at index 0 when above_row_id is
/// None). If above_row_id isn't in the order (defensive; shouldn't
/// happen when the event was built from this order), inserts at the
/// end.
pub fn apply_row_move_event(order: &[usize], event: &RowMoveEvent) -> Vec<usize> {
    let mut out: Vec<usize> = order
        .iter()
        .copied()
        .filter(|&id| id != event.moved_row_id)
        .collect();
    let insert_at = match event.above_row_id {
        None => 0,
        Some(above) => match out.iter().position(|&id| id == above) {
            Some(p) => p + 1,
            None => out.len(),
        },
    };
    out.insert(insert_at, event.moved_row_id);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn midpoints(order: &[usize], height: f32) -> Vec<(usize, f32)> {
        // Each row is `height` tall, laid out from y = 0 downward.
        // Midpoint of row i is (i as f32 + 0.5) * height.
        order
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, (i as f32 + 0.5) * height))
            .collect()
    }

    #[test]
    fn slot_is_the_number_of_midpoints_above_the_dragged_center() {
        // Five rows, 100px tall each. Midpoints at 50, 150, 250, 350, 450.
        let mids = midpoints(&[0, 1, 2, 3, 4], 100.0);
        // Drag center at 0 — nothing is above. Slot 0.
        assert_eq!(insertion_slot_for_dragged_center(&mids, 0.0), 0);
        // Drag center at 50 — nothing is strictly less than 50. Slot 0.
        assert_eq!(insertion_slot_for_dragged_center(&mids, 50.0), 0);
        // Drag center at 100 — midpoint 50 is below. Slot 1.
        assert_eq!(insertion_slot_for_dragged_center(&mids, 100.0), 1);
        // Drag center at 449 — midpoints 50, 150, 250, 350 are below. Slot 4.
        assert_eq!(insertion_slot_for_dragged_center(&mids, 449.0), 4);
        // Drag center past everything — slot equals len.
        assert_eq!(insertion_slot_for_dragged_center(&mids, 1000.0), 5);
    }

    #[test]
    fn slot_with_variable_heights() {
        // Rows at different heights: 0: y 0-20 (mid 10),
        // 1: y 20-120 (mid 70), 2: y 120-125 (mid 122.5).
        let mids = [(0, 10.0), (1, 70.0), (2, 122.5)];
        assert_eq!(insertion_slot_for_dragged_center(&mids, 5.0), 0);
        assert_eq!(insertion_slot_for_dragged_center(&mids, 50.0), 1);
        assert_eq!(insertion_slot_for_dragged_center(&mids, 100.0), 2);
        assert_eq!(insertion_slot_for_dragged_center(&mids, 500.0), 3);
    }

    #[test]
    fn moving_down_past_one_row() {
        let order = vec![0, 1, 2, 3];
        // Moving row 0 to after row 1. Others = [1, 2, 3]. Slot 1
        // lands after others[0]=1, i.e. between 1 and 2.
        let event = build_row_move_event(&order, 0, 1).unwrap();
        assert_eq!(event.moved_row_id, 0);
        assert_eq!(event.above_row_id, Some(1));
        assert_eq!(event.below_row_id, Some(2));
        let new_order = apply_row_move_event(&order, &event);
        assert_eq!(new_order, vec![1, 0, 2, 3]);
    }

    #[test]
    fn moving_up_past_one_row() {
        let order = vec![0, 1, 2, 3];
        // Moving row 3 to after row 0. Slot 1 among others [0, 1, 2].
        let event = build_row_move_event(&order, 3, 1).unwrap();
        assert_eq!(event.above_row_id, Some(0));
        assert_eq!(event.below_row_id, Some(1));
        let new_order = apply_row_move_event(&order, &event);
        assert_eq!(new_order, vec![0, 3, 1, 2]);
    }

    #[test]
    fn moving_to_the_very_top() {
        let order = vec![0, 1, 2, 3];
        let event = build_row_move_event(&order, 2, 0).unwrap();
        assert_eq!(event.above_row_id, None);
        assert_eq!(event.below_row_id, Some(0));
        let new_order = apply_row_move_event(&order, &event);
        assert_eq!(new_order, vec![2, 0, 1, 3]);
    }

    #[test]
    fn moving_to_the_very_bottom() {
        let order = vec![0, 1, 2, 3];
        // Slot 3 = past the three other rows [1, 2, 3].
        let event = build_row_move_event(&order, 0, 3).unwrap();
        assert_eq!(event.above_row_id, Some(3));
        assert_eq!(event.below_row_id, None);
        let new_order = apply_row_move_event(&order, &event);
        assert_eq!(new_order, vec![1, 2, 3, 0]);
    }

    #[test]
    fn noop_slot_returns_none() {
        let order = vec![0, 1, 2, 3];
        // Row 1 sits after exactly one other row (row 0). Slot 1 means
        // it lands in the same place.
        assert!(build_row_move_event(&order, 1, 1).is_none());
        // Row 0 sits after zero other rows. Slot 0 = same place.
        assert!(build_row_move_event(&order, 0, 0).is_none());
        // Row 3 sits after three other rows. Slot 3 = same place.
        assert!(build_row_move_event(&order, 3, 3).is_none());
    }

    #[test]
    fn slot_out_of_range_is_refused() {
        let order = vec![0, 1];
        // Only one "other row" for a 2-row order. Slot 2 is past the
        // end of that list, so build returns None.
        assert!(build_row_move_event(&order, 0, 2).is_none());
    }

    #[test]
    fn moving_a_row_past_a_left_only_row_is_legal() {
        // Rows 0 and 2 have right slots; row 1 is LeftOnly (no right
        // slot). The math doesn't care — LeftOnly rows are ordinary
        // rows for ordering purposes. Move row 0 to the bottom of the
        // three-row order (after row 2).
        let order = vec![0, 1, 2];
        let event = build_row_move_event(&order, 0, 2).unwrap();
        assert_eq!(event.above_row_id, Some(2));
        assert_eq!(event.below_row_id, None);
        let new_order = apply_row_move_event(&order, &event);
        assert_eq!(new_order, vec![1, 2, 0]);
    }

    #[test]
    fn moving_a_left_only_row_is_legal_in_the_math() {
        // Same again, this time the LeftOnly row itself is the one
        // being moved. The component never offers a handle for it, so
        // this scenario shouldn't happen in the UI — but the math is
        // correct if it did.
        let order = vec![0, 1, 2];
        let event = build_row_move_event(&order, 1, 0).unwrap();
        assert_eq!(event.above_row_id, None);
        assert_eq!(event.below_row_id, Some(0));
        let new_order = apply_row_move_event(&order, &event);
        assert_eq!(new_order, vec![1, 0, 2]);
    }

    #[test]
    fn single_row_order_has_no_valid_move() {
        let order = vec![7];
        // Slot 0 = same place (row has zero rows before it).
        assert!(build_row_move_event(&order, 7, 0).is_none());
        // Slot 1 = past the end, also refused because len is 0.
        assert!(build_row_move_event(&order, 7, 1).is_none());
    }

    #[test]
    fn unknown_row_id_returns_none() {
        let order = vec![0, 1, 2];
        assert!(build_row_move_event(&order, 99, 1).is_none());
    }

    #[test]
    fn apply_is_idempotent_given_the_same_event() {
        // Applying the same event twice to the same starting order
        // produces the same result.
        let order = vec![0, 1, 2, 3];
        let event = build_row_move_event(&order, 0, 2).unwrap();
        let once = apply_row_move_event(&order, &event);
        let twice = apply_row_move_event(&order, &event);
        assert_eq!(once, twice);
    }

    #[test]
    fn random_sequence_of_moves_keeps_order_a_permutation() {
        let mut state: u64 = 0x9e3779b97f4a7c15;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        let n = 10usize;
        let mut order: Vec<usize> = (0..n).collect();
        for _ in 0..200 {
            let moved = order[(next() as usize) % n];
            let slot = (next() as usize) % n;
            if let Some(event) = build_row_move_event(&order, moved, slot) {
                order = apply_row_move_event(&order, &event);
            }
            let mut sorted = order.clone();
            sorted.sort_unstable();
            assert_eq!(
                sorted,
                (0..n).collect::<Vec<_>>(),
                "order stopped being a permutation"
            );
        }
    }

    #[test]
    fn move_and_move_back_restores_the_original_order() {
        let original = vec![0, 1, 2, 3, 4];
        // Move row 1 to the end.
        let ev1 = build_row_move_event(&original, 1, 4).unwrap();
        let moved = apply_row_move_event(&original, &ev1);
        assert_eq!(moved, vec![0, 2, 3, 4, 1]);
        // Move it back to slot 1 (its original position among others
        // in the new order: [0, 2, 3, 4], slot 1 = after 0).
        let ev2 = build_row_move_event(&moved, 1, 1).unwrap();
        let restored = apply_row_move_event(&moved, &ev2);
        assert_eq!(restored, original);
    }
}
