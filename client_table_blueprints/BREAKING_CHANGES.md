# Breaking changes

## 2026-10-20: every_block_in_existence.position became frac_position

The position column on every_block_in_existence was a Real that held the
row's own id as a float. That was insertion order, not sort order, so it
couldn't express a reorder.

Replaced with frac_position: Text, a hex string produced by
smart_frac_index::frac_core. Two blocks compare by this string to decide
which comes first, with block id as the tie-break.

Rows created before this change have no frac_position. The project is
pre-data and gets wiped, so no migration was written.

## 2026-10-20: PositionToInsert.SpecificPosition removed

yrs's PositionToInsert is now AtEnd or Between { block_id_of_above:
Option<String>, block_id_of_below: Option<String> }. SpecificPosition
held a numeric slot in the underlying array, which stopped meaning
anything once order came from frac position.

Same signature change applies to every builder that took a
PositionToInsert (add_block). Callers that want "put it after the last
thing" pass AtEnd; callers that want "put it between these two blocks"
pass Between with the neighbour block ids.