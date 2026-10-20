# cache and preload

Separate from the visual work. Technical feature.

## load order

A page loads from three places, in this order: preload, offline, server.

Preload is the first blocks already in memory from predictive loading. Offline is the local db - always read first, regardless of connectivity. Server is asked only after the first two, and only when connected.

## page states

Three variants, as an enum: PreloadState, OfflineLoadedState, OnlineState.

PreloadState is when only the preloaded prefix is loaded. OfflineLoadedState is when the page is fully loaded from local db and hasn't been checked against the server yet. OnlineState is when it has been checked and is up to date.

A page is not usable for applying edits until it is OnlineState. Edits from other clients land in OnlineState, so that is the natural boundary.

## resuming from preload

The page continues from where the preload stopped. It does not restart from the beginning. The count of preloaded rows is the offset for the next read.

## going online

Ask the server for the page's hash. If it matches local, transition to OnlineState without transferring the page. If it doesn't match, ask for the whole page and replace the local copy. That replacement is the current answer, not the final one; a reconcile path comes later.

## predictive loading

Built now, not retrofitted later. Preload the first blocks of pages ahead of navigation. The db library needs a skip/offset method so the preload count can be passed as the offset for the next read.

## ownership

A struct created in main owns the load path and is passed around. Callers talk to that struct; the load order above is internal to it.

## status

Design decided. Not built.