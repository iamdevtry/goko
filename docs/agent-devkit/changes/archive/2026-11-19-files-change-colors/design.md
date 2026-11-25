# Files change decorations: status colours in the tree and dots on tabs

## Goal

The Files section shows which checkout files changed, the way VS Code's explorer does: a changed
file's name in the tree is tinted by its Git status, and each open-file tab carries a status dot,
conflicts included. Today the Files tree colours only Git-ignored rows (dimmed), and a tab shows
only the unsaved `•`, so the user must switch to Changes to learn what changed.

Requested by the user on 2026-11-19: "thêm highlight kiểu VS Code: tên file trong cây Files đổi
màu theo status, và tab file có chấm màu modified/conflict".

## Scope

In scope: tinting file rows in the Files tree by Git status, a status dot on open-file tabs, the
status word in each tab's accessible label, and the tone mapping shared with the Changes rows.

Out of scope: a periodic Changes read while the Files section shows (decision D1), status letters
or `+N −N` counts in the tree, tinting folder rows, decorations in the search-results list,
changes to how Git-ignored rows look (they stay dimmed), a colour setting, and any backend,
protocol, schema, settings or dependency change.

## Behavior

**Status source.** Marks are derived at render time from `EditorState.changes.data`, and only when
`changes.checkout` equals `editor.checkout_key` — the freshness guard `changed_layer` already
uses. Nothing is persisted and no Git, backend or LSP request is issued by this feature (D1).
Tone colours are read from the frame's `Ink`, so every theme recolours the marks.

**Per-path status priority.** conflict → unstaged → staged → untracked. The tree describes the
working tree (what the open file reads from disk), so an unstaged edit outranks an older staged
view of the same path. `changed_layer` keeps its own priority (conflict → staged → unstaged →
untracked) for "Show changes", resolve actions and the doc bar; that behavior is unchanged.

**Tones.** The same status reads the same in the tree, on a tab and in the Changes list.

| Status | Tone |
|---|---|
| `modified`, `typechange` | `ink.warning` |
| `added`, `untracked` | `ink.success` |
| `deleted` | `ink.danger` |
| `renamed`, `copied` | `ink.accent` |
| a conflict entry (any conflict kind) | `ink.danger` |
| any other/unknown status | no mark |

**Files tree.** A file row with a mark sets `TreeItem::label_color(tone)`. Git-ignored rows
(`FileMark::Ignored`, `FileMark::IgnoredDir`) keep `ink.muted`, so an ignored path never shows a
change mark. Folders, unmarked files and the rest of the tree keep today's label colour. No
letter badge, dot or diff count is added to tree rows.

**Tabs.** A tab whose path has a mark shows a 6×6 dot (radius 3, 5 px gap) before the existing
dirty marker, filled with the tone. The dot is `a11y_hidden`; the status word rides in the tab's
accessible label instead: `"{label}, {status}"`, then `", unsaved"` when the tab is dirty, so a
changed and dirty tab reads `"{label}, {status}, unsaved"` (today: `"{label}, unsaved"`). The
active and inactive tabs both show it; a clean unchanged tab renders no dot and its label text is
exactly today's. The existing `•` unsaved marker keeps its place and tone.

**Freshness.** Marks change only when Changes data changes. The existing cadence applies: the
Changes list is re-read on window focus, on entering the Editor, on checkout change, and every
three seconds while the Changes section shows. While the user stays in Files, tints can lag until
one of those edges; no periodic Changes read is added (D1). The open document's existing disk
check is independent and untouched.

## Edge cases

- Dirty plus changed: both indicators show (status dot in the status tone, `•` in warning).
- A path staged as added and then edited again: the unstaged `modified` tone wins (warning).
- A path both conflicted and listed as unstaged: the conflict mark wins (danger).
- An ignored path that a stale change entry names (should not happen with fresh data): it stays
  dimmed; ignored styling always wins.
- A renamed entry tints its new path (`entry.path`); `orig_path` is not tinted.
- A deleted file that is not in the tree but is open as a tab: the tab still shows the danger dot.
- Changes data for another checkout, or no data yet: no tint and no dot anywhere.
- Switching theme recolours marks without a reload, because tones resolve from the current `Ink`.
- The search-results aside, an empty tree, and the Changes/History/Stashes sections render exactly
  as today.

## Interfaces

- `apps/rune/src/editor/changes.rs`: new `pub(crate) fn status_tone(status: &str, ink: &Ink) ->
  Option<rune::Color>` owning the table above; `file_item` tints through it for the statuses
  `name_tint` already covers, so Changes rows look unchanged.
- `apps/rune/src/editor/files.rs`: `fn fresh_changes(s: &State) -> Option<&GitViewChanges>`
  extracted from `changed_layer` (same behavior); `fn change_marks(s: &State, ink: &Ink) ->
  HashMap<String, (rune::Color, &'static str)>` deriving one mark per changed path (tone and
  `status_label` text) with the priority above; `files_view` builds it once per render and passes
  it to `tab_bar`/`doc_tab` and `tree_pane`.
- No new `EditorState` field, no persistence, no rune-kit change (it reuses
  `TreeItem::label_color`), and no backend, protocol, schema, settings or dependency change.

## Verification

- `changes.rs` unit tests: each status maps to its tone; unknown statuses map to `None`.
- `files.rs` state tests: priority (conflict > unstaged > staged > untracked), freshness (another
  checkout's data yields no marks), and renamed paths.
- `files.rs` rune-harness tests: tree label colours per status via `ElementInfo::text_color`;
  ignored rows stay muted; dot `fill` per status by `doc-tab-change:{path}` key; no dot for a
  clean or foreign-checkout tab; accessible tab labels.
- `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace`.
- App check on the fresh `target/Goko.app` with `agent-desktop` (Goko window only): a modified,
  an added, a deleted and a conflicted file show their colours in the tree; their tabs show the
  matching dot and the accessible label names the status; switching theme recolours them.

## Related context

- [rune-editor.md](../../../../llm/domains/rune-editor.md) (`RNE-section-views`, `RNE-editor-tabs`,
  `RNE-changes-auto-refresh`, `RNE-changes-git-writes`).

