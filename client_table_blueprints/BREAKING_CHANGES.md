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