# Import plan (nov 25)

## Problem
Import and `GetAllYrsPages` both want to hand over every page in one reply. With enough pages the client has to hold all of it in memory at once.

## Decision: pagination, not streaming
- Client asks for "up to N pages after this page_id" (no cursor for the first request).
- Server replies with the pages and a `has_more` flag.
- Client loops until `has_more` is false.
- Server stays stateless. The client owns the cursor, so resuming after a dropped connection is just asking again from the last page it got.
- Every request still gets exactly one reply, so the queue and the router don't change.

## Why not streaming
Streaming means many replies per request. That would touch the queue (single oneshot reply), the router (one waiter per message id) and disconnect handling. More work than this needs, and we don't need continuous data.

## Scope
- Import is built on pagination from the start.
- `GetAllYrsPages` is left alone for now. If it becomes too big, rewrite it the same way as import.

## To do
- Add cursor and limit to the request payload, `has_more` to the reply payload.
- Build import on top of that.
- Write a test that imports and exports with a lot of pages.
