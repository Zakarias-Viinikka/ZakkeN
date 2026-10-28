# feel_labs — third pass, 2026-10-29

Durable record of the brief this pass was built from. Kept so a future
AI (or a future you) can see what was decided, in what order, and what
was out of scope. If the code and this file disagree, the code wins;
update this file when the shape changes.

The brief below is the exact instruction handed to the AI that built
this pass, pasted verbatim. Implementation decisions beyond it, and
anything cut, live in that AI's final report (not stored here). The
project's own living context lives in STORED_AI_CONTEXT.md — this file
is a point-in-time spec, not a replacement for it.

---

Third pass on feel_labs, a Leptos 0.8 CSR playground for deciding how typing and editing effects feel. Read STORED_AI_CONTEXT.md first, then src/main.rs, styles/app.css, and (for the motion reference) src/drag_reorder_lab/cascade.rs. Also skim BRIEF_2026-10-28_visual_pass.md for how the previous pass was structured — that's history, not active instructions.

11 labs, dark slate + periwinkle identity, shared tokens and primitives in app.css. The drag_reorder_lab port is the reference for what satisfying block motion feels like. Do not touch it, or anything under cqrs_state_synchronization/web_interface.

Six things, in order. ./local_cargo.sh check after each.

1. Main menu icons.

Pen wiggle: only the pen body should animate. The underline stroke under the pen must stay still. Currently the whole <svg> rotates.

Eye: clicking should close slowly and stay closed. Clicking again opens slowly. The closed state is the disabled signifier — drop the diagonal slash. Keep the row dimming.

2. Running into the wall. The bar currently reads as a rigid rectangle translating. Replace it with a line whose ends dissolve — soft alpha edges, ink materialising and dissolving. No hard-edged bar, no rectangle fading as a whole. Direction is yours: my recommendation is to drop the leftward travel entirely (line appears at the wrap row already soft-edged, holds briefly, dissolves), but pick the version you think feels right. Comment the choice.

3. Search bar.

Grow the corpus to ~14 so some queries overflow.

Give the results card a scrollbar.

Arrival and removal must be animated with distinct feels — new rows rise/pop in, leaving rows fall/fade out. The mount animation today (rise 4px + fade) is one of the two; make the other visibly different, and make removal feel finished. Keep the existing demo, debounce, scoring, Recent / "no matches" states.

4. Deleting a block. Current red-hold-fade-pop stays as variant 1. Add two more via a segmented control (Fall/Spring/Ripple pattern from Block cascade). Your invention for 2 and 3. Preserve the "row is removed only after the animation finishes" model — no instant removal. Semantic arc is still "marked for deletion → leaves → list reflows".

5. Idle page. Rework entirely. Three alternatives via segmented control:

(a) the current idea, better executed;

(b) and (c) are yours to invent — genuinely different ideas, not just different easing.
Honor prefers-reduced-motion. Every alternative must slide/return cleanly on input. The user has given you full latitude here.

6. Organic split. Reframe as a block splitting into a new block — the animation should read as a slate/periwinkle .mock-block splitting into two blocks. It does not need to connect to any creation flow. Whether the two end up stacked, side by side, or one emerges below is yours. It must feel like a real, finished motion, not a blob. The interface stays a single "Split" button (plus reset if useful); no new controls.

Constraints from the project. (See STORED_AI_CONTEXT.md gotchas — they're accurate.) Highlights: stylist blocks only style their root element; turbofish needs a let; restart CSS animations by remounting via a generation counter in For; on_cleanup needs handles with clear()/remove(); honor prefers-reduced-motion; no ancestor of a lab may create a stacking context (new_page_popup's fixed overlay must keep working). Do not add persistence, backend, routing, or carried state. Do not change the picker's unmount-on-switch. Do not change any existing lab's interface beyond the two segmented controls I asked for. Do not touch drag_reorder_lab/ or cqrs_state_synchronization/web_interface. Use the existing tokens and primitives; do not introduce a parallel colour or easing vocabulary.

Motion reference. --ease-out for quiet slides, --ease-spring for spring feel. drag_reorder_lab/cascade.rs is the gravity-slide reference; Block cascade (Fall) mirrors it. Anything new should sit comfortably next to those.

Docs at the end. Update STORED_AI_CONTEXT.md (labs, gotchas, the new icon logic, the new motion curves, the idle page redesign). Update README.md for the affected labs. Write a new dated BRIEF_2026-10-29_third_pass.md in the same style as the first one, capturing this brief verbatim, so the pattern continues.

Final report: one paragraph per item on what you built and the decisions you made, plus anything you cut. Also call out any motion curve you invented (name + formula) so a future pass can find it.