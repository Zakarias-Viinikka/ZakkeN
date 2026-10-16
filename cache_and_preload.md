# cache and preload

Separate from the visual work. Technical feature.

Pages should be loadable into memory before they're needed. Not the
whole page — just the first blocks.

A page needs to be able to load from three places: the cache
(preloaded, in memory), memory (already loaded, maybe partially),
and the real db. The page logic has to handle all three.

## status

Not started. Written down so it isn't forgotten.
