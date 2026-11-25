# Backlinks Plan

## Architectural Decision
Backlinks are not an independent CRDT or a separate synced database table. 

The block text content inside pages is the sole source of truth. Links to other pages are stored directly in block content as:
[[PageNav|<target_page_id>]]

## Current Strategy: On-Demand Query
To show backlinks for a page, query every_block_in_existence directly:

    SELECT page_id, content FROM every_block_in_existence 
    WHERE content LIKE '%[[PageNav|' || :target_page_id || ']]%';

### Why this approach
- Zero sync overhead: Backlinks sync automatically because page content already syncs via Yrs and love letters.
- No state divergence: Avoids conflicts where a backlink record disagrees with the actual text in the page.
- Export/Import simplicity: Backlinks don't require any dedicated export/import tables or logic; importing block text restores all links.

## Future Optimization (If Needed)
For personal wikis (thousands of blocks), SQLite string matching takes single-digit milliseconds. If the dataset ever grows large enough (50,000+ blocks) that full-table scans cause lag, we can introduce a local projection index table (source_page_id, target_page_id) populated on save/merge without touching network sync.
