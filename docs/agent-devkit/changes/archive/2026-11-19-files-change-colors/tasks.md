# Files change decorations tasks

## Design

[Files change decorations](design.md)

## Global Constraints

- Marks derive only from `EditorState.changes.data` when `changes.checkout == editor.checkout_key`; no new Git, backend or LSP read, no polling change, no persisted state (D1).
- Display priority per path is conflict → unstaged → staged → untracked. `changed_layer`'s existing priority (conflict → staged → unstaged → untracked) and its callers stay unchanged.
- Tones come from the frame's `Ink`: `modified`/`typechange` warning, `added`/`untracked` success, `deleted` danger, `renamed`/`copied` accent, any conflict danger, unknown none.
- Git-ignored rows stay `ink.muted` and never take a mark; folder rows are never tinted.
- A tab's dot is `a11y_hidden`, and its accessible label names the status with `unsaved` last.
- No backend, protocol, schema, settings, dependency, or rune-kit change.

## Impact map

Entry: `apps/rune/src/editor/files.rs::files_view` (renders `tree_pane`, `tab_bar`, `doc_tab`), `apps/rune/src/editor/files.rs::tree_pane`, `apps/rune/src/editor/files.rs::doc_tab`

Flow: worker Changes reply → `apps/rune/src/editor/backend/apply.rs` sets `EditorState.changes.data` and `changes.checkout` → `files_view` → `change_marks` → `TreeItem::label_color` in `tree_pane` and the dot plus accessible label in `doc_tab`; `changed_layer` keeps feeding the doc bar's Show changes and resolve actions

State changes: none; marks are derived per render and dropped with the frame; no persistence

External effects: none found — no new Git child process, backend request, LSP message, file write, network call or settings read

Change candidates: `apps/rune/src/editor/files.rs`, `apps/rune/src/editor/changes.rs`, `CHANGELOG.md`, `docs/llm/domains/rune-editor.md`

Verification: `cargo test -p goko-rune` (the module tests named per task), `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, and the `agent-desktop` app check of the fresh `target/Goko.app`

Verified at: 7c11febdba6af2bb95a1e1765f1b484399eee30f

## Tasks

### Task 1 — Shared status tone

Files: `apps/rune/src/editor/changes.rs::status_tone` (new), `apps/rune/src/editor/changes.rs::file_item`, `apps/rune/src/editor/changes.rs::tests`

Interfaces: produces `pub(crate) fn status_tone(status: &str, ink: &Ink) -> Option<rune::Color>`; consumes `Ink`; consumes the existing `name_tint` and `status_letter` helpers; produces no visible change to Changes rows.

Change: own the tone table in one place — `modified` and `typechange` → `ink.warning`, `added` and `untracked` → `ink.success`, `deleted` → `ink.danger`, `renamed` and `copied` → `ink.accent`, every other status → `None`. `file_item` keeps tinting only the statuses `name_tint` covers, now through `status_tone`, so its rendered colours and trailing letters are unchanged.

Verify: `cargo test -p goko-rune changes::tests` — a new test asserts each documented status tone and `None` for an unknown status; the existing `file_item`/row tests still pass.

Progress: done

Result: `cargo test -p goko-rune changes::tests` — the new `status_tone_covers_each_documented_status` and all existing Changes tests pass; the full `cargo test -p goko-rune` run reports 972 passed, 0 failed.

### Task 2 — Derive per-path marks

Files: `apps/rune/src/editor/files.rs::fresh_changes` (new), `apps/rune/src/editor/files.rs::changed_layer`, `apps/rune/src/editor/files.rs::change_marks` (new), `apps/rune/src/editor/files.rs::tests`

Interfaces: consumes `changes::status_tone` and `changes::status_label` (Task 1), `EditorState.changes.data` and `changes.checkout`; produces `fn fresh_changes(s: &State) -> Option<&app_core::git_view::GitViewChanges>` and `fn change_marks(s: &State, ink: &Ink) -> HashMap<String, (rune::Color, &'static str)>`.

Change: extract `changed_layer`'s freshness check into `fresh_changes` and keep `changed_layer` behavior identical on top of it. `change_marks` returns an empty map without fresh data; otherwise one entry per changed path with the display priority conflict → unstaged → staged → untracked, a conflict entry always taking `ink.danger` and its `status_label` word, and every other entry taking `status_tone`; an entry whose status has no tone (unknown) contributes no mark. A path listed in several groups appears once, with the highest-priority mark.

Verify: `cargo test -p goko-rune files::tests` — new tests: a path in conflicts and unstaged maps to danger with the conflict's label ("Both modified"); a path in staged and unstaged maps to the unstaged status's tone; a path only untracked maps to success; a renamed entry marks its new path and never its `orig_path`; data whose `checkout` differs from `checkout_key`, and missing data, both yield an empty map. The existing `changed_layer_priority_and_checkout_freshness` test still passes untouched.

Progress: done

Result: `cargo test -p goko-rune files::tests` — `change_marks_priority_freshness_and_renames` proves conflict > unstaged > staged > untracked, a renamed entry's new path, and empty marks for foreign or missing data; `changed_layer_priority_and_checkout_freshness` still passes.

### Task 3 — Tint changed file names in the Files tree

Files: `apps/rune/src/editor/files.rs::files_view`, `apps/rune/src/editor/files.rs::tree_pane`, `apps/rune/src/editor/files.rs::tests`

Interfaces: consumes `change_marks` (Task 2) and `TreeItem::label_color` from `rune_kit`; produces the marks value passed to the tab strip in Task 4.

Change: build the marks once per `files_view` render and hand them to `tree_pane`. In the tree row closure, a non-folder node whose path has a mark sets `TreeItem::label_color(tone)`; `FileMark::Ignored` and `FileMark::IgnoredDir` rows keep the existing `label_color(dim)`/`icon_color(dim)` and never take a mark; every other row keeps the tree's normal label colour. No badge, dot, or count is added to tree rows.

Verify: `cargo test -p goko-rune files::tests` — a rune-harness test with fresh Changes data (modified `a.rs`, untracked `new.txt`, deleted `gone.txt`, conflicted `README.md`, unchanged `guide.md`, and an ignored `.env` also named untracked) asserts each changed name's `ElementInfo::text_color` equals its tone, the unchanged name's colour is none of the four tones, `.env` stays `ink.muted`, and a folder row (`src`) carries no tone. The existing ignored-tree and filter harness tests still pass.

Progress: done

Result: `cargo test -p goko-rune files::tests` — `files_tree_tints_changed_names_and_keeps_ignored_dimmed` shows the warning/success/danger tones, the conflict in danger, the ignored `.env` muted despite a stale untracked entry, and unchanged files plus a folder plain.

### Task 4 — Status dot and accessible label on tabs

Files: `apps/rune/src/editor/files.rs::files_view`, `apps/rune/src/editor/files.rs::tab_bar`, `apps/rune/src/editor/files.rs::doc_tab`, `apps/rune/src/editor/files.rs::tests`

Interfaces: consumes the marks from Task 3; produces the tab strip's status dot element keyed `doc-tab-change:{path}` and the extended accessible label.

Change: pass the marks into `tab_bar`/`doc_tab`. A tab with a mark renders a 6×6, radius-3 dot before the existing dirty marker, filled with the tone, `.a11y_hidden()`, keyed `doc-tab-change:{path}`; a tab without a mark renders no dot. The tab's `.label(...)` appends `, {status}` from the mark's `status_label` text, keeping `, unsaved` last, so a changed dirty tab reads `"{label}, {status}, unsaved"` and a clean one keeps today's text; the dot and label use the same mark for active and inactive tabs. `help` stays the full path.

Verify: `cargo test -p goko-rune files::tests` — a rune-harness test opens `a.rs` (modified), `b.rs` (clean), `c.rs` (conflict "Both modified"), `d.rs` (modified and dirty) and a deleted-only `e.rs` with fresh Changes data and asserts `doc-tab-change:*` fills (warning, none, danger, warning, danger), labels (`"a.rs, Modified"`, `"b.rs"`, `"c.rs, Both modified"`, `"d.rs, Modified, unsaved"`, `"e.rs, Deleted"`), and the existing dirty-marker elements still render. A second assertion with Changes data for another checkout shows no `doc-tab-change:*` element and today's labels. The existing tab tests still pass.

Progress: done

Result: `cargo test -p goko-rune files::tests` — `tabs_show_status_dots_and_name_the_status` proves the warning/danger dots, no dot for a clean or foreign-checkout tab, labels `a.rs, Modified` / `c.rs, Both modified` / `d.rs, Modified, unsaved` / `e.rs, Deleted`, and the `•` marker.

### Task 5 — Wiki and changelog

Files: `docs/llm/domains/rune-editor.md`, `CHANGELOG.md`

Interfaces: consumes the delivered behavior and evidence paths from Tasks 1–4 and this change's `delta.md`.

Change: call `document-wiki`'s `## Task updates` to merge the delta's `RNE-change-marks` block into `docs/llm/domains/rune-editor.md`, and refresh any page text it contradicts — in particular `RNE-editor-tabs`' list of what a tab shows and `RNE-section-views` prose about section reads (no new read is added). Add a user-facing `### Added` entry under `## [Unreleased]` in `CHANGELOG.md`: file names in the Files tree now take their Git status colour, open-file tabs show a matching colour dot (conflicts included), the unsaved `•` stays, and the colours follow the Changes data the editor already refreshes.

Verify: `test -f` every Evidence path in the new wiki block; the four-backtick block equals the wiki block; `grep` finds no sentence claiming the Files tree has no change colours; `cargo fmt --all -- --check` and `cargo test --workspace` pass.

Progress: done

Result: `RNE-change-marks` merged into `docs/llm/domains/rune-editor.md`; `CHANGELOG.md` has the `[Unreleased]` entry. Wiki validator: 0 errors in rune-editor.md (the 166 pre-existing errors in desktop-theme.md and rune-chrome.md remain); `cargo fmt --all -- --check` and `git diff --check` pass. Real-app check of the fresh debug build (`GOKO_HOME=/tmp/goko-verify-files`, X11 `:1`, window screenshots only): the Files tree tints `CHANGELOG.md`, `changes.rs` and `files.rs` orange (modified) and `delta.md`/`design.md`/`tasks.md` green (untracked); the `CHANGELOG.md` and `changes.rs` tabs show orange status dots while the clean `branch.rs` tab shows none. Limits: that checkout has no deleted or conflicted sample and the theme was not switched, and `agent-desktop` a11y-tree inspection is macOS-only; those cases rest on the harness tests.

## Files inspected, no change

- `apps/rune/src/editor/state.rs` — `EditorState.changes` fields and `checkout_key`; confirms data plus checkout identity and that no state field is added.
- `apps/rune/src/editor/backend/apply.rs` — where a Changes reply stores `data` and `checkout`; read to confirm marks can derive at render without an update hook.
- `apps/rune/src/editor/workspace.rs` — section dispatch to `files_view`; no change needed.
- `crates/rune-kit/src/tree/view.rs` — `TreeItem::label_color`, `dot` and the a11y-hidden trailing content; colouring reuses it unchanged.
- `apps/rune/src/theme.rs` — the `Ink` palette fields used for tones; read only.
- `apps/rune/src/editor/autorefresh.rs` — the existing three-second Changes cadence and the Files-section check; unchanged by D1.
- `docs/llm/domains/rune-editor.md` — requirement IDs and current text read for the delta (`RNE-section-views`, `RNE-editor-tabs`, `RNE-changes-auto-refresh`).

## Approval Gate

Status: approved

## Decision Log

### D1 — Freshness without a new Git read

Question: Should the Files section keep its change decorations live by re-reading Changes every three seconds while it shows?

Decision: No. Marks use the freshest Changes data the editor already holds for the checkout, so they refresh on the existing edges — window focus, Editor entry, checkout change, and while the Changes section polls. The user chose the lowest-memory, lowest-CPU option after the three candidates were compared (no new process, no retained derived state).

Impact: the design's Freshness behavior; `RNE-changes-auto-refresh` stays as written; Task 2 derives marks per render instead of caching them.

Confirmed by user: 2026-11-19

