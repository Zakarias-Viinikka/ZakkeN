the create_page could be written to call 3 methods:
...
insert_to_pages_tbl()
insert_to_every_block_tbl()
insert_to_uncommitted_diffs()
...

## Steps

1. add deps
2. create_page function (already made)
3. create the yrs_wrapper. insert two blocks.
4. snapshot it.
5. snapshot the "version" thingy.
6. figure out the yrsactivepages thingy, serialize it to blob
7. create the actual row we'll insert
8. update the actual every block thingy
9. figure out uncommitted_diffs

---

a new page makes a yrs doc and creates two blocks in "every block table" the
title block and the first block.

version in the page table is the blob of yrs version vector thingy.

i need to also update the active_pages tbl. since there's a new page.

page_status. a separate yrs doc just for keeping track of if a page is
disabled or not.

we need to update the actual uncommitted diffs. that is part of it too.


want a verification folder or rs file for generalized methods to check if
stuff is in sync