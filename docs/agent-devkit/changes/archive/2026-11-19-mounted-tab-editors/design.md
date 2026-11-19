# Mounted tab editors: switching tabs keeps each file's code surface alive

## Goal

Switching between open file tabs is immediate even for large files. Today the editor pane builds one code
surface for the active document, so every switch binds a different `TextEdit` to the same field element and
Rune lays out the whole document text again. Measured on the release build, switching between two ~3 100-line
files blocks one commit for **78–85 ms** (text layout 72–77 ms); in the debug build the same switch blocks
**1.7–2.8 s**. Small files stay under 16 ms, which is why only large (often changed) files feel laggy. Tinting
syntax spans over the whole file accounts for about 70% of that layout.

This change keeps one code surface per open tab mounted while its tab is open, and shows only the active one.
Each document's text is laid out once, when it first opens; a later switch reuses that layout instead of
rebuilding it. Data flow is unchanged: activation still re-arms highlight, blame, line changes and the disk
check.

## Scope

In scope: a mounted code surface per open tab, active-only chrome and callbacks, focus transfer between the
active and parked documents, inactive surfaces being invisible, unpickable and unfocusable, and the
measurement that proves the switch no longer lays out the document.

Out of scope: changes to Rune itself (lazy or virtualized text layout), tab persistence, a cap on open tabs or
their retained layout memory, changes to when highlight/blame/line-change reads run, and any backend, protocol,
schema or settings change.

## Behavior

**Surface stack.** `editor_pane` mounts one code surface per path in `EditorState.tabs.order`, all in the same
rectangle. The active tab's surface is opaque, focusable and pickable; every other surface is
`.opacity(0.0)` (Rune skips fully transparent subtrees when drawing and picking) and `.a11y_hidden()`, with its
field `.focusable(false)`. The global chrome (tab strip, doc bar, section notices) stays once, above the stack.

**Per-document binding.** `Code`'s accessors become `Rc<dyn Fn(&S) -> &CodeState>` and
`Rc<dyn Fn(&mut S) -> &mut CodeState>` instead of function pointers, so each surface can select its own
document by path (`EditorState::doc_code(path)`, `doc_code_mut(path)` — the active document or the parked one).
Only the active surface is built with the editor callbacks (`on_changed`, `on_save`, `on_go_to`, indent, send,
secondary press) and the chrome (`conflict_bar`, `find_bar`, notice and blame-error rows, `footer`); an inactive
surface builds only its scroll viewport, gutter, change bars and field, so it has no live controls and cannot
act on state.

**Focus and caret.** Activating a tab sets the new document's `code.edit.focused = true` and clears it on the
parked document, keeping the existing rule that a dirty saved document stays focused after a save. Caret,
selection, undo history, scroll offsets and find state stay per document, as they already do.

**Data flow.** `after_active_change` keeps its current behavior: re-arm highlight, request blame and line
changes, and probe the disk. Language-server mirroring stays active-document-only. The mounted but inactive
surfaces only retain their rendered text and layout; they receive no highlight results and no reads.

**Cost.** The first open of a document still lays out its text once (as today). A switch between two open tabs
must not lay out either document's text again: on the release build, a switch between two ~3 000-line documents
must produce no commit over 16 ms (today: 78–85 ms). Window resize may still re-lay out every mounted document
because their text is width-wrapped; that cost is measured and reported, not removed here.

## Edge cases

- A tab opens for the first time: its surface lays out its text once, exactly as opening does today; the
  active chrome and callbacks are built for it.
- Closing a tab unmounts its surface and drops its retained layout; the next switch keeps its own layout.
- A doc with no text surface (binary/media opened externally) mounts no surface; switching to it keeps today's
  pane.
- Switching while the code field has focus moves focus to the newly active document, including to a read-only
  document (which then has no caret field).
- Switching to a dirty parked document shows its buffer, caret and undo history as today; no save or disk check
  is issued for it beyond the existing activation probe.
- Inactive surfaces must not answer the pointer, keyboard, Tab order or assistive technology; the active one
  alone is pickable and focusable.
- Many open tabs or several large documents raise memory (one retained layout each). No new cap is added;
  closing tabs is the existing control.
- Window resize re-lays out every mounted document once; the change records that cost rather than hiding it.
- A theme switch rebuilds the surfaces; text layout is reused because the content is unchanged.
- A checkout switch clears the tabs, so every surface unmounts with them.

## Interfaces

- `apps/rune/src/editor/code.rs`: `Code::new`/`Code::build` take `Rc<dyn Fn(&S) -> &CodeState>` and
  `Rc<dyn Fn(&mut S) -> &mut CodeState>`; a new `Code::chrome(bool)` (or an equivalent `build` flag) makes an
  inactive surface skip the callbacks and chrome.
- `apps/rune/src/editor/state.rs`: `EditorState::doc_code(&self, path) -> Option<&CodeState>` and
  `doc_code_mut(&mut self, path) -> Option<&mut CodeState>` select the active document or the parked one.
- `apps/rune/src/editor/files.rs`: `editor_pane` builds the per-tab surface stack and marks the active tab;
  tab strip and doc bar stay as they are.
- No Rune, backend, protocol, schema, settings or dependency change.

## Verification

- Harness tests: with two tabs open, both surfaces are mounted (one element per path), only the active one is
  visible, pickable and focusable; switching moves focus and leaves both surfaces in the tree; typing edits the
  active document and never the inactive one; find and conflict bars exist only on the active surface; closing a
  tab removes its surface; the existing tab, focus, find, conflict, indent, go-to and blame tests pass.
- Real app, release build with `RUNE_COMMIT_BUDGET_MS=16`: switching between `changes.rs` (3 119 lines) and
  `history.rs` (3 176 lines) produces no commit over 16 ms after the change (baseline 78–85 ms); the first open
  of each file and a window resize are measured and reported separately.
- `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`.

## Related context

- [rune-editor.md](../../../../llm/domains/rune-editor.md) (`RNE-editor-tabs`, `RNE-code-surface`,
  `RNE-changes-auto-refresh`).

## Previous work

- [Editor tabs](../2026-11-17-editor-tabs/design.md) — the tab model this change keeps mounted.
- [Files change decorations](../2026-11-19-files-change-colors/design.md) — the marks that made the
  larger changed files the common case; the lag itself is size-driven and predates them.

