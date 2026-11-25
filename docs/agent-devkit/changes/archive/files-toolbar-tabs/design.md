# Files toolbar tabs: one header in the file-tree aside

## Goal

The Files section's filter and content-search controls move out of two full-width
rows above the whole editor and into one compact header owned by the file-tree
aside: a `Files | Text | Context` tab strip with one input under it. The code
editor gets back the two rows (69 px) and its full width; the filter and the
search box stop competing for one wide row and instead fill the aside directly
above the tree; and the four file action buttons become one `File actions` menu
button on the strip.

## Scope

In:

- A header inside the Files aside: row 1 holds the three-segment tab strip and
  the actions button; row 2 holds one input and, on `Context`, the index status.
- Tab semantics for the two existing inputs (`files.filter`, `files.search_edit`)
  and for the search pane's `open` flag.
- The header's four file actions (New File…, New Folder…, Refresh, Collapse All)
  in a menu anchored to the actions button.
- Entry-point alignment: `⌘F`, `⇧⌘F`, the go-to-definition fallback, and a file
  filter edit each land on the tab they act on.
- Widget tests for the header, its tabs, the hint state, the status line, the
  menu, and aside-width overflow; the `docs/llm/domains/rune-editor.md` delta for
  `RNE-content-search` and `RNE-concept-search`; `ARCHITECTURE.md` and
  `CHANGELOG.md`.

Out:

- Search backends, `MIN_QUERY`, the 250 ms debounce, result grouping, the
  truncation note, and result-row navigation (`j`/`k`) stay exactly as they are.
- The aside's width range and its persisted width (`pane_layout`).
- Persisting the tab or the search mode across restarts (the mode stays a
  session value, as today).
- Moving the Changes, History, Stashes, or Servers toolbars.
- Any new request, protocol message, dependency, or setting.

## Behavior

**Layout.** `files_view` becomes a column of the confirmation overlays and one
row: `[aside | grip | editor]`. The aside column holds the header above the pane
it swaps (`tree_pane` or `search_results`), keeps its width, its `PANE_MIN`
shrink and its right border. No part of the header spans the editor.

- Header row 1: `row`, full aside width, 30 points tall, 8-point side padding,
  6-point gap, bottom border. Its children are one `SegmentedControl` labelled
  `Files view` with segments `Files`, `Text`, `Context`, filling the row, and one
  28×28 icon-only button (key `files-actions-button`, role `button`, label and
  tooltip `File actions`).
- Header row 2: `row`, 32 points tall, same padding and gap, holding one
  `TextInput` that fills the row (`min_w 90`, shrinkable) and, only while the
  `Context` tab is active, the index status text from `status_line`
  (`min_w 0`, shrinkable, middle-truncated).
- The tab strip fills the row so its three segments share the aside width
  equally; a long label truncates in the middle rather than widening the strip.

**Tab identity.** The selected tab is a projection of state, not a new field:
`Files` when `files.search.open` is false, `Text` when it is true and
`files.search.mode == SearchMode::Content`, `Context` when it is true and the
mode is `SearchMode::Concept`. Label and placeholder therefore always agree.

**Input per tab.** One `TextInput` is mounted, bound to the active tab:

| Tab | Edit | Placeholder | `on_change` |
|---|---|---|---|
| `Files` | `files.filter` | `Filter files` | `on_filter_change` (rebuild the model, `open = false`) |
| `Text` | `files.search_edit` | `Search text` | `search::on_query_change` |
| `Context` | `files.search_edit` | `Search by context` | `search::on_query_change` |

Both edits keep their text across tab switches; switching never clears either.
The field's accessible name stays its placeholder (no separate `.label`).

**Picking a tab.** A press on `Files` sets `search.open = false`, moves focus to
`files.filter` with its caret at the end, and shows the tree already rebuilt from
the current filter text (no extra rebuild). A press on `Text` or `Context`:

1. calls `search::set_mode` with that mode (unchanged: it clears the previous
   mode's `results`, `error`, `query`, `pending` and `searching`, re-arms the
   existing input when it holds at least `MIN_QUERY` characters, and starts an
   index pass for `Context` when none finished);
2. sets `search.open = true`;
3. moves focus to `files.search_edit` with its caret at the end;
4. when the mode was already the picked one, `results` is `None`, `pending` is
   `None`, `error` is `None`, and the trimmed text holds at least `MIN_QUERY`
   characters, calls `on_query_change` to arm the 250 ms debounce.

Existing `results` are shown at once with no new request.

**Search pane versus tree.** `search.open` now means "the aside shows the search
pane":

- The pane shows from the moment a search tab is picked, also with no query — it
  then shows the existing `Type at least 2 characters` text.
- A trimmed query below `MIN_QUERY` clears `results`, `pending`, `error` and
  Concept's `searching`, and leaves the pane showing (the tab no longer jumps to
  the tree while the user clears the box).
- Editing the file filter, and a checkout switch (`search::reset`), still show
  the tree.
- A query of at least `MIN_QUERY` characters is issued exactly as today
  (debounce, `poll`, `open` already true).
- With no `results` yet, the pane's text follows the input: `Type at least 2
  characters` while the trimmed text is shorter than `MIN_QUERY`, and
  `Searching…` once it is long enough (the debounce or the reply is in flight).

**Entry points.** `⌘F` (`keyboard::find` → `focus_section_filter`) selects the
`Files` tab before focusing the filter with its text selected. `⇧⌘F`
(`keyboard::open_content_search`) forces `Content` mode, sets `open = true`, and
focuses the search input with its text selected, so the pane is visible before
the query lands. The go-to-definition fallback already forces `Content` and sets
`open = true`, so it lands on `Text`. A doc opened from a result keeps the pane.

**File actions menu.** The actions button opens a menu titled `files-actions`
anchored to `files-actions-button` with, in order: `New File…`, `New Folder…`,
a separator, `Refresh`, `Collapse All`. Each item closes the menu before acting.
`New File…` and `New Folder…` use `selected_dir` exactly as the removed buttons
did; `Refresh` calls `refresh_files`; `Collapse All` calls
`tree_get_mut(st).collapse_all()`. The menu is a kit layer mounted beside the
existing `files::menu_layer` in `main.rs`, only in the Editor view.

**Keyboard and assistive access.** The strip keeps `SegmentedControl`'s
`focus_group` and `role="radiogroup"` with group label `Files view`; each segment
is a `radio` labelled by its own text and exposes its selected state. The actions
button is a focusable `button` labelled `File actions`. At 180 points of aside
width a segment label truncates in the middle and the input keeps its 90-point
minimum; nothing in the header overflows the aside.

## Edge cases

- **Short or cleared query on a search tab:** the tab stays selected, the pane
  shows `Type at least 2 characters`, no request is issued, and the tree is not
  shown.
- **Search error showing:** picking the same search tab again does not re-issue
  (the `error` blocks the re-arm); editing the box does. The error text stays in
  the pane.
- **Valid query before the reply:** with enough characters typed and no reply
  yet, the pane reads `Searching…`, never the short-query hint.
- **Index status:** shown only on the `Context` tab, from `status_line`
  (`Indexing done/total files`, `Indexed the first N files`,
  `Context search unavailable: <reason>`), middle-truncated when narrow.
- **Filter text and search text both set:** picking `Files` shows the tree
  filtered by the kept filter text; picking `Text` or `Context` shows the kept
  results (or the hint) with the kept search text.
- **`⇧⌘F` while `Context`:** forces `Content` (as today), so the strip shows
  `Text` and the literal results.
- **No checkout / empty tree:** `tree_pane`'s loading, error and `No files`
  states are unchanged, with the header still mounted.
- **New File prompt while the menu is open:** the menu closes on item selection
  and the sidebar prompt mounts above the header (it is layered after the file
  menus in `main.rs`).
- **Aside dragged to `PANE_MIN` (180):** labels truncate, the input keeps its
  minimum, and no header child extends past the aside's right edge.
- **Checkout switch:** the tree shows, the search mode is kept, the search text
  is kept, and results are cleared — the same rule as today, now expressed as the
  `Files` tab being selected.

## Interfaces

- `apps/rune/src/editor/state.rs::FilesState` gains
  `pub actions_open: bool` (the actions menu is open at its button).
- `apps/rune/src/editor/state.rs::search_state::Search::open` keeps its name; its
  doc comment becomes "the aside shows the search pane".
- `apps/rune/src/editor/files.rs`:
  - `aside_header(s: &State, ink: &Ink, k: &Kit) -> El` replaces `toolbar` and
    `search_row`;
  - `tab_index(s: &State) -> usize` (0 Files, 1 Text, 2 Context);
  - `pick_tab(st: &mut State, tx: &mut Tx, index: usize)`;
  - `actions_menu_layer(s: &State, layer: &mut Vec<El>) -> Option<El>`;
  - `toolbar_icon` is renamed `action_button(icon, label, key, ink, on_press)` for
    its single remaining caller (the actions button);
  - `mode_switch` is deleted (the strip replaces it).
- `apps/rune/src/editor/keyboard.rs::focus_text_end(edit: &mut TextEdit)` is the
  caret-at-end variant of the existing private `focus_text`; `pick_tab` uses it
  after `clear_edit_focuses`.
- Unchanged signatures: `search::set_mode`, `search::on_query_change`,
  `search::reset`, `files::on_filter_change`, `files::refresh_files`,
  `files::selected_dir`, `pane_layout::current`.

## Verification

- `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`,
  `cargo test --workspace`.
- `apps/rune/src/editor/files.rs` tests: the strip and one input mount with
  `Files` selected and `Filter files`; picking `Text` swaps to `Search text`
  without losing either text; typing and letting `poll` run replaces the tree
  with results; picking `Files` restores the filtered tree; picking `Text` again
  shows the stored results without issuing a query; clearing the box on a search
  tab keeps the tab and shows the hint; a valid query shows `Searching…` before
  the reply; picking `Context` sets the mode,
  `Search by context`, and renders each index status; the actions menu lists its
  four items and `New File…` opens the New Entry prompt; every header element
  stays inside the aside at 280 and 180 points.
- `apps/rune/src/main.rs` tests: the 760-point no-horizontal-overflow checks
  still pass with the header inside the aside, and the editor surface keeps its
  width below the header's old rows.
- `apps/rune/src/editor/keyboard.rs` and `search.rs` tests: `⌘F` returns to the
  `Files` tab, `⇧⌘F` opens the pane in `Text` before the query lands, and a
  shortened query keeps the pane.
- A real-app check per `AGENTS.md`: a uniquely named copy of `target/Goko.app`
  driven with `agent-desktop` on the Goko window only — pick each tab, type in
  each input, open the actions menu, and drag the aside to 180 px.

## Related context

- [Rune editor](../../../../llm/domains/rune-editor.md) — `RNE-content-search` and
  `RNE-concept-search` are the requirements this change modifies.

## Previous work

- [Concept search: a ranked, indexed code search in the Files search box](../2026-11-13-concept-search/design.md)
  and its [tasks](../2026-11-13-concept-search/tasks.md) established the
  always-visible search row, the `Text | Context` switch, and the `open` flag
  this change reshapes.

