# STORED_AI_CONTEXT - rapier_tests_combined

Durable knowledge for a fresh AI working on this crate. Assumes the workflow file is separate and already known. Assumes the workspace-root `STORED_AI_CONTEXT.md` is available and does not repeat it - this one specializes it.

This file was written before the crate was built. Treat the "should" tense as intent, not observed state. If the code and this file disagree, the code wins and this file should be corrected.

## What this crate is

`rapier_tests_combined` merges two working siblings.

From `rapier_test_2`: the drag-reorder UI, the global vacuum, the visual confirmation system, the rapier query wrapper.

From `rapier_collision_test`: the cascade motion - velocity ramp, propagation from the drop point down, target derived from the block ahead rather than from own position.

It does **not** carry over from `rapier_collision_test`: the arrow-based click input, the three-region box layout, the refusal mode, the seniority value, the boundary logic, or the click queue. All of those were prototype-specific to the collision demo and are irrelevant here. Only the motion math survives.

## The two-page model

The crate renders two page panels side by side, both visible at once. Both pages' blocks live in one shared rapier world, positioned at their absolute stage coordinates. There is no cross-page code path. A drag between pages is one continuous motion through the shared coordinate space; a drop between pages is one query against the same world.

Blocks do not store which page they belong to. Page membership is derived from x position, and the rapier world holds the truth. Do not duplicate this as state.

This is the entire reason the crate exists - the shared physics world is what lets the system say "a block moved from this document to that one, to this location, and used to be here."

## Rapier is never stepped

Same as every sibling in this workspace. Fixed bodies. Query engine only. The world answers "what is nearest to this point" and "what is above and below this rect". Motion is DOM-driven; positions live in a leptos signal and styles read from it.

## The cascade

On drop, the dragged block snaps to its slot instantly. Every block from the drop index downward gets a new target. Every block whose target differs from its current position animates toward it with a velocity ramp, in the same shape as `rapier_collision_test`'s `tick`: accelerate up to a cap, snap to target on arrival.

The cascade propagates downward from the drop point. Blocks above the drop point do not move.

Targets are computed as `bottom_of_block_above + GAP`, chained from the drop point down. That is the "triggerer's target + own height + gap" pattern from `rapier_collision_test`, generalized to variable heights. It is what gives flush stacking with no gaps and no overshoot. It is the single most important line in the cascade. If displaced blocks are overshooting or leaving gaps, this is where to look.

## Variable block heights

No fixed step. Every block has its own height. `GAP` is fixed and shared between both pages. Page top offset is `EDGE_PAD`.

## Drop during a running cascade

There is no queue. A new drop recomputes everything from the new arrangement; in-flight movers retarget to whatever target the new arrangement gives them. New arrangement wins immediately.

## Vacuum

One global vacuum on the left edge of the stage, outside both page panels. Same semantics as `rapier_test_2`: drag a block onto it to store, drag the small rect out and release over a page to insert, second store overwrites the first, release the rect outside any page and it snaps back. The rect's drop query uses the stored block's full footprint, not the rect's visual size.

## Visual confirmation

Copy of `rapier_test_2`'s. `FourLines` renderer, `Confirmation::One` / `Many`, four-colour palette by index, `viz_on` checkbox gating every `set` and clearing on uncheck. Nothing here is new.

## The stale capture trap

`<For>` reuses child views when the key matches. Anything captured at render time goes stale forever after that item's data changes. Style closures and event handlers must read live values out of the signal, not the captured ones. See the workspace-root `STORED_AI_CONTEXT.md` for the exact shape of the bug - it bit twice in `rapier_test_2`.

## Environment gotchas

`chmod` requires a real shell. `leptosNew` scaffolds have no `.cargo/config.toml`; trunk builds them anyway, bare `cargo` inside them does not. `zap`'s `mv` refuses directories; use `/bin/mv` in a real shell.

## The root forwarder

Root `local_cargo.sh` decides what `c r` from the workspace root serves. It `cd`s into one member and execs that member's own `local_cargo.sh`. As of the last edit it pointed at `rapier_collision_test`; it should now point at `rapier_tests_combined`.

## Source material

`rapier_test_2/` - the working reorder / vacuum / confirmation crate. Copy `rapier_world.rs` and `visual_confirmation.rs` as-is; extend `rapier_world.rs` only for the drop-point query (given `(x, y)` and an exclusion set, return the page and the nearest block index).

`rapier_collision_test/` - the working cascade motion. Port `propagate` and the `tick` velocity ramp. Strip everything else.

## Open decisions

These were left to the building AI to pick and state rather than ask about. If they were not written down anywhere after the fact, they are worth re-deciding cleanly:

- Page widths, gap between pages, vacuum position and size at a 1600-wide viewport.
- How many blocks per page at startup. Default was a handful, so the cascade is visible on first load.
- Whether the dragged block floats above everything with `z-index` or sits inside the stage. Default was floating, since the drag is transient.
- Page membership as derived state vs stored state. Derived from x is the correct answer - do not add a `page` field to the block struct.
