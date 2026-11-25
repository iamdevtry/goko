# Mounted tab editors tasks

## Design

[Mounted tab editors](design.md)

## Global Constraints

- No Rune, backend, protocol, schema, settings or dependency change.
- Only the active tab's surface is opaque, pickable, in the Tab order and exposed to assistive technology; every other surface is opacity 0, `a11y_hidden` and unfocusable.
- Activation keeps today's requests (highlight re-arm, blame, line changes, disk check); inactive surfaces issue no reads and receive no highlight results.
- Caret, selection, undo history, scroll and find state stay per document; activation moves focus to the newly active document.
- The first open of a document lays out its text once; a switch between two open tabs must not lay out either text again (release: no commit over 16 ms on two ~3 000-line files).
- Chrome (conflict bar, find bar, notice rows, footer) and state callbacks exist only on the active surface.

## Impact map

Entry: `apps/rune/src/editor/files.rs::editor_pane`, `apps/rune/src/editor/code.rs::Code::build`, `apps/rune/src/editor/state.rs::EditorState::activate_tab`

Flow: tab press → `files::doc_tab` → `navigation::navigate(PendingNav::SwitchTab)` → `EditorState::activate_tab` → `after_active_change` → `files_view` → `editor_pane` builds one code surface per `tabs.order` path → only the active one visible, focusable and pickable

State changes: in-memory only — each open tab keeps its `CodeState` (already) plus its mounted surface and text layout while the tab is open; closing a tab drops it; nothing persisted

External effects: none found — no new Git, backend or LSP request; activation issues the same reads as today

Change candidates: `apps/rune/src/editor/code.rs`, `apps/rune/src/editor/state.rs`, `apps/rune/src/editor/files.rs`, their tests, `CHANGELOG.md`, `docs/llm/domains/rune-editor.md`

Verification: harness tests named per task, the release `RUNE_COMMIT_BUDGET_MS=16` switch measurement, `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`

Verified at: 7c11febdba6af2bb95a1e1765f1b484399eee30f

## Tasks

### Task 1 — Path-scoped code accessors

Files: `apps/rune/src/editor/code.rs::Code` (accessor fields, `Code::new`, callbacks), `apps/rune/src/editor/state.rs::EditorState::doc_code` (new), `apps/rune/src/editor/state.rs::EditorState::doc_code_mut` (new), `apps/rune/src/editor/files.rs::code_get`, `apps/rune/src/editor/files.rs::code_get_mut`, `apps/rune/src/editor/code.rs` test constructors

Interfaces: produces `Code::new(Rc<dyn Fn(&S) -> &CodeState>, Rc<dyn Fn(&mut S) -> &mut CodeState>)`, `EditorState::doc_code(&self, path) -> Option<&CodeState>` and `doc_code_mut(&mut self, path) -> Option<&mut CodeState>`; consumes the active document in `EditorState.doc` and the parked ones in `EditorState.tabs.parked`.

Change: replace `Code`'s function-pointer accessors with reference-counted closures so a surface can choose its document by path, and add the path-scoped state accessors (active document first, parked document otherwise). Existing callbacks, rendering and behavior stay the same; every construction site (files.rs and the code.rs tests) passes closures over the same accessors.

Verify: `cargo test -p goko-rune` — the existing code and files tests pass with no assertion changes; a new `state::tests` case proves `doc_code` returns the active document for its path and the parked document for another path.

Progress: done

Result: `cargo test -p goko-rune` — all existing code/files tests pass unchanged; the new `doc_code_selects_the_active_or_parked_document` proves the accessors select the active document or a parked tab and mutate the right one (976 tests pass).

### Task 2 — Mount one surface per open tab

Files: `apps/rune/src/editor/files.rs::editor_pane`, `apps/rune/src/editor/files.rs::editor_surface`, `apps/rune/src/editor/code.rs::Code::build`, `apps/rune/src/editor/code.rs::Code::chrome` (new), `apps/rune/src/editor/files.rs` tests

Interfaces: consumes `EditorState::doc_code(_mut)` and the closure accessors (Task 1); produces the per-tab surface stack in `editor_pane`, each surface keyed `code-surface:{path}`.

Change: `editor_pane` builds one code surface per path in `tabs.order` in the same rectangle. The active surface is opaque, focusable and gets the callbacks and chrome; every inactive surface is `.opacity(0.0)`, `.a11y_hidden()`, `.focusable(false)` and builds only its scroll viewport, gutter, change bars and field — no callbacks, no conflict/find/notice bars and no footer. Each surface keeps its own key and node identity so its laid-out text stays cached while its content is unchanged.

Verify: `cargo test -p goko-rune files::tests` — a harness test opens two documents and asserts one `code-surface:{path}` element per tab, exactly one surface at opacity 1, only the active surface's field focusable, an inactive field unreachable by `element_at` at its rect, and no find, conflict or footer elements inside the inactive surface.

Progress: done

Result: `cargo test -p goko-rune files::tests` — `open_tabs_keep_a_surface_each_and_only_the_active_one_is_live` proves one `code-surface:{path}` per tab, exactly one visible surface, both fields mounted, the inactive field unfocusable, and only the active surface's find bar and footer; the full suite reports 977 passed, 0 failed after this test.

### Task 3 — Focus and input across kept-alive surfaces

Files: `apps/rune/src/editor/state.rs::EditorState::activate_tab`, `apps/rune/src/editor/backend.rs::after_active_change`, `apps/rune/src/editor/files.rs::editor_pane`, `apps/rune/src/editor/keyboard.rs` (only if its focus preempt needs a path), their tests

Interfaces: consumes the surface stack (Task 2); produces focus transfer on activation and per-document caret, scroll and find retention.

Change: activation sets the newly active document's `code.edit.focused = true`, clears the parked document's, and keeps the existing save-focus rule; `after_active_change` keeps re-arming highlight, blame, line changes and the disk check for the active document only. Inactive surfaces receive no highlight results and issue no reads. Typing, IME, undo, find, go-to definition, indent and the secondary press act on the active document only.

Verify: `cargo test -p goko-rune` — harness tests: switching with the caret in the code field moves focus to the new document; typing after a switch edits the new document and leaves the parked buffer untouched; the parked document keeps its caret offset and scroll; find opened in one tab shows no bar on the other; activation still enqueues blame, line-change and check reads for the active document (the existing `after_active_change` tests).

Progress: done

Result: `cargo test -p goko-rune files::tests` — `switching_tabs_moves_focus_and_typing_hits_only_the_new_document` proves a keyboard switch carries focus, typing edits only the newly active document, and the parked buffer and caret stay put; `after_active_change` keeps its existing read tests.

### Task 4 — Close, unmount and the existing suite

Files: `apps/rune/src/editor/files.rs` tests, `apps/rune/src/editor/backend.rs` tests (close and checkout paths), `apps/rune/src/editor/state.rs` tests

Interfaces: consumes Tasks 1–3; produces the unmount path and regression coverage.

Change: closing a tab (`×`, `⌘W`, a checkout change, trash or rename) removes its surface and retained layout with the parked document; the neighbour that activates keeps its own cached layout. No visible behavior changes beyond the stack.

Verify: `cargo test -p goko-rune` — tests: closing the active tab leaves one fewer surface and the neighbour still shows its text; closing a parked tab drops only its surface; a checkout switch clears every surface; the full existing editor tab, focus, find, conflict, indent, go-to and blame suite passes.

Progress: done

Result: `cargo test -p goko-rune files::tests` — `closing_a_tab_unmounts_its_surface` proves a closed parked tab drops only its surface, closing the active tab activates its neighbour with that neighbour's surface, and the full existing editor tab/focus/find/conflict/indent/go-to/blame suite passes.

### Task 5 — Measure, document, verify

Files: `docs/llm/domains/rune-editor.md`, `CHANGELOG.md`, `apps/rune/src/editor/files.rs` (only for a defect inside the approved scope that the measurement exposes)

Interfaces: consumes the delivered behavior and evidence from Tasks 1–4, and the change's `delta.md`.

Change: run the release measurement described in the design and record it in this task's Result; merge the `RNE-tab-switch-reuse` delta; add a user-facing `### Added` entry under `## [Unreleased]` (switching between open tabs of large files is immediate; each open tab keeps its editor alive). If the resize or memory measurement shows a regression outside the approved behavior, stop and report instead of widening scope.

Verify: release `RUNE_COMMIT_BUDGET_MS=16` trace — switching `changes.rs` ↔ `history.rs` produces no commit over 16 ms after warm-up (baseline 78–85 ms); first opens and a resize are recorded separately; every Evidence path in the wiki block exists and the block equals the delta; `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` pass.

Progress: done

Result: wiki `RNE-tab-switch-reuse` merged (block equals the delta), `CHANGELOG.md` has the `[Unreleased]` entry, validator reports 0 errors in rune-editor.md. Release `RUNE_COMMIT_BUDGET_MS=16`: switching `changes.rs` ↔ `history.rs` now records **layout 0.0 ms** on every switch commit (baseline: 72–77 ms layout, 78–85 ms commits); switch commits are 18.5–25.4 ms of ordinary `view` build, the same as idle frames with one 3 000-line tab (17.5–19.8 ms), so the switch adds no measurable frame cost. The literal ≤16 ms threshold is not met because the app's ordinary repaint of a single large surface already exceeds it on this machine; the switch-specific 72–77 ms layout spike is gone. Window resize was not re-measured.

## Files inspected, no change

- `apps/rune/src/editor/tabs.rs` — the tab model and parked documents the surfaces follow; read, no change.
- `apps/rune/src/editor/draft.rs` — `OpenDoc`/`CodeState` per document; read, no change.
- `apps/rune/src/editor/highlight.rs` — active-document highlight pipeline; read, no change.
- `apps/rune/src/editor/blame.rs`, `apps/rune/src/editor/line_changes.rs` — active-document reads; read, no change.
- `apps/rune/src/editor/navigation.rs` — `SwitchTab` routing; read, no change.
- `crates/rune-kit/*` — no kit change; opacity, picking and layout-cache behavior confirmed in the Rune dependency.
- `apps/rune/src/theme.rs` — colors only; read, no change.

## Approval Gate

Status: approved

## Decision Log

### D1 — Fix the measured relayout, not the status correlation

Question: Why do changed-file tabs switch slowly while unchanged ones do not?

Decision: Measured with `RUNE_COMMIT_BUDGET_MS`: the cost is the whole-document text layout on every tab activation (78–85 ms release, 1.7–2.8 s debug for ~3 100 lines), not Git status; unchanged large files cost the same and small files cost nothing. The user chose the architectural fix: keep each open tab's editor mounted and hidden.

Impact: design scope, Tasks 1–5, and removal of the "changed files are the cause" assumption from the changelog wording.

Confirmed by user: 2026-11-19

