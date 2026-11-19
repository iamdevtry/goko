# Files toolbar tabs tasks

## Design

[Files toolbar tabs: one header in the file-tree aside](design.md)

## Global Constraints

- The Files aside header shows exactly three segments `Files`, `Text`, `Context`
  and exactly one input; no header child may extend past the aside's right edge
  at any aside width from 180 to 480 points.
- `search.open` means "the aside shows the search pane": a trimmed query below
  `MIN_QUERY` never closes that pane, and editing the file filter always shows
  the tree.
- The mounted input's placeholder is exactly `Filter files`, `Search text`, or
  `Search by context`, matching the selected tab, and its accessible name stays
  that placeholder.
- No new backend request, protocol message, dependency, or persisted setting;
  `MIN_QUERY` (2), the 250 ms debounce, `SegmentedControl`, and `pane_layout`
  keep their current values and APIs.

## Impact map

Entry: `apps/rune/src/editor/files.rs::files_view`, mounted by
`apps/rune/src/editor/workspace.rs:431` and `apps/rune/src/editor/keyboard.rs:852`.

Flow: `files_view` → `toolbar`/`search_row` → `TextInput`/`SegmentedControl`;
`on_filter_change` → `rebuild_file_model`; `search::on_query_change` →
`search.pending` → `search::poll` → `search.open` + `Req::Search`;
`navigation::go_to_search_reply` → `open = true`;
`keyboard::{find, open_content_search}` → `focus_section_filter` /
`open_content_search` → the two `TextEdit`s.

State changes: UI only — `State.editor.files.filter`, `search_edit`, and
`search.{open, mode, pending, results, error}`; the mode lasts for the app
session and the aside width is persisted by `pane_layout` (both unchanged). No
storage or protocol change.

External effects: existing `Req::Search`, `Req::FileList`, and
`Req::CreateEntry` jobs through the editor backend worker; no new jobs.

Change candidates: `apps/rune/src/editor/files.rs`,
`apps/rune/src/editor/state.rs`, `apps/rune/src/editor/search.rs`,
`apps/rune/src/editor/keyboard.rs`, `apps/rune/src/main.rs`,
`docs/llm/domains/rune-editor.md`, `ARCHITECTURE.md`, `CHANGELOG.md`.

Verification: `cargo fmt --all -- --check`,
`cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, plus the
real-app check in `design.md` `## Verification`.

Verified at: 7c11febdba6af2bb95a1e1765f1b484399eee30f

## Tasks

### Task 1 — Search pane semantics and the keyboard entry points

Files: `apps/rune/src/editor/search.rs::on_query_change` and its tests,
`apps/rune/src/editor/keyboard.rs::open_content_search`,
`apps/rune/src/editor/keyboard.rs::focus_section_filter` and the tests in that
module, `apps/rune/src/editor/state.rs::search_state::Search` (doc comment only).

Interfaces: Produces `Search.open` with the meaning "the aside shows the search
pane" for Task 2's tab projection. Consumes nothing from other tasks.

Change: In `on_query_change`, the branch for a trimmed query under `MIN_QUERY`
clears `pending`, `results`, `error`, and Concept's `searching`, and no longer
sets `open` — the pane stays on whichever tab the user chose. In
`open_content_search` (`⇧⌘F`), set `s.editor.files.search.open = true` after
`force_content` and before the existing focus and `on_query_change` call, so the
pane is visible before the debounce lands. In `focus_section_filter`, for
`Section::Files`, set `s.editor.files.search.open = false` before focusing
`files.filter`, so `⌘F` returns to the tree tab. `search::reset` keeps setting
`open = false` on a checkout switch, and `files::on_filter_change` keeps setting
it on a filter edit. Rewrite `Search::open`'s doc comment to "The aside shows the
search pane instead of the tree."

Verify: `cargo test -p goko-rune editor::search` and
`cargo test -p goko-rune editor::keyboard` pass, including
`short_query_keeps_the_pane_open` (set `open = true`, one-character input, call
`on_query_change` → `open` stays true, `results`/`pending`/`error` cleared),
`shift_cmd_f_opens_the_pane_in_text_mode` (Concept mode + text → `open` true,
mode Content, `search_edit` focused, `pending` armed),
`cmd_f_returns_to_the_files_tab` (`open = true` then the Files `⌘F` path →
`open` false, `filter` focused), `checkout_switch_shows_the_tree_and_keeps_the_mode`
(`open = true` in Concept mode then `search::reset` → `open` false, mode still
Concept), and every existing assertion that a cleared box
sets `open = false` updated to the new rule.

Progress: done
Result: `on_query_change` keeps `search.open` on a short query and clears `query`/`searching` so a reply for the abandoned text is dropped; `poll` no longer sets `open`, so a debounced query landing after the user picked `Files` or edited the filter cannot pull the pane back; `⇧⌘F` sets `open` before the debounce; the Files `⌘F` clears it; `Search::open`'s doc comment is the pane meaning; `mounted_editables` lists only the header's mounted input. `cargo test -p goko-rune editor::` passed (648 tests) including `short_query_keeps_the_pane_open`, `an_issued_query_never_pulls_the_aside_back_to_the_results`, `shift_cmd_f_opens_the_pane_in_text_mode`, `cmd_f_returns_to_the_files_tab`, `only_the_mounted_input_is_in_the_focus_list_and_rows_need_the_pane`, and `a_checkout_switch_shows_the_tree_keeps_the_mode_and_requests_a_pass`.

### Task 2 — The aside header: tab strip, one input, actions menu

Files: `apps/rune/src/editor/files.rs::files_view`,
`apps/rune/src/editor/files.rs::toolbar`,
`apps/rune/src/editor/files.rs::search_row`,
`apps/rune/src/editor/files.rs::mode_switch`,
`apps/rune/src/editor/files.rs::toolbar_icon` (renamed `action_button` for its
one remaining caller),
new `apps/rune/src/editor/files.rs::aside_header`,
`apps/rune/src/editor/files.rs::search_results` (empty-state text only),
`apps/rune/src/editor/files.rs::tab_index`,
`apps/rune/src/editor/files.rs::pick_tab`,
`apps/rune/src/editor/files.rs::actions_menu_layer`,
`apps/rune/src/editor/state.rs::FilesState` (`actions_open`),
`apps/rune/src/main.rs` (mount the actions layer beside `files::menu_layer`), and
the tests in `apps/rune/src/editor/files.rs`.

Interfaces: Consumes Task 1's `Search.open` and the existing `Search.mode`,
`files.filter`, `files.search_edit`, `search::set_mode`, `search::on_query_change`,
`files::on_filter_change`, `files::refresh_files`, `files::selected_dir`,
`keyboard::clear_edit_focuses`. Produces `aside_header`, `tab_index`, `pick_tab`,
`actions_menu_layer`, `action_button`, and `FilesState.actions_open`.

Change: `files_view` keeps its dialogs and its `[aside | grip | editor]` row but
drops the top-level `toolbar` child; the aside column becomes
`col().w(width).h_full().shrink().min_w(PANE_MIN).border_sides(right)` with
`aside_header(s, ink, k)` above the existing `search_results`/`tree_pane` child
(which keeps `w_full().h_full()` and `min_h(0)`). `aside_header` returns a column
of two rows: row 1 (`h(30)`, `px(8)`, `gap(6)`, `items_center`, bottom border)
holds a `SegmentedControl::new(&["Files", "Text", "Context"], Some(tab_index))`
labelled `Files view`, `.width(None)` so the segments split the aside, whose
`on_pick` calls `pick_tab`, followed by one 28×28 actions button; row 2 (`h(32)`,
same padding and gap) holds one `TextInput` bound to the active tab's
`TextEdit`/placeholder/`on_change` exactly as `design.md`'s table says, built
with `.width(None)`, `.grow(1.0)`, `.shrink()`, `.min_w(90.0)`, followed on the
`Context` tab by `status_line(&search.index)` as `theme::txt(..., T11M_LH, …)`
with `.min_w(0.0)`, `.shrink()`, `.truncate_middle()` and the failure colour.
`tab_index` returns 0 when `!search.open`, else 1 for `SearchMode::Content` and 2
for `SearchMode::Concept`. `pick_tab(st, tx, index)` follows `design.md`'s
"Picking a tab" steps, using `keyboard::clear_edit_focuses` and the new
`keyboard::focus_text_end` (add it beside `focus_text`: `focused = true` and
`set_selection(len, len)`), and calls `on_query_change` only under the stated
guard. The actions button is built by `action_button` (the renamed
`toolbar_icon`: 28×28, `role("button")`, hover/press, focus ring, help), labelled
`File actions`, keyed `files-actions-button`, and its `on_press` sets
`st.editor.files.actions_open = true`; `actions_menu_layer` builds
`Menu::new("files-actions").anchor("files-actions-button").open(true)` with
`New File…` (`theme::FILE_PLUS`, `ask_for_new(st, &selected_dir(st), false)`),
`New Folder…` (`theme::FOLDER_PLUS`, folder `true`), a separator, `Refresh`
(`theme::REFRESH`, `refresh_files`), and `Collapse All` (`theme::COLLAPSE_ALL`,
`tree_get_mut(st).collapse_all()`); each item sets `actions_open = false` first
and `.on_close` clears it. `search_results`'s empty state reads `Type at least 2
characters` while the trimmed input is shorter than `MIN_QUERY` and `Searching…`
otherwise, so a valid query never shows the short-query hint. Delete `toolbar`,
`search_row`, and `mode_switch`;
update the module doc comment at the top of `files.rs` to describe the header in
the aside. Replace the tests
`the_search_row_is_always_visible_with_a_text_context_switch` and
`search_input_types_and_arms_debounce` with the cases below, and change
`the_aside_follows_the_box_edited_last` to pick the `Text` tab before typing the
query and the `Files` tab before typing the filter.

Verify: `cargo test -p goko-rune editor::files` passes, including these cases:
`the_aside_header_mounts_the_tab_strip_and_one_input` (radiogroup `Files view`
with radios `Files`/`Text`/`Context`, `Files` selected, `textboxes(&rt).len() == 2`,
the `Filter files` textbox present and no `Search text` one);
`picking_text_swaps_the_input_and_keeps_both_texts` (type `edit` on Files, pick
`Text` → `Search text` mounted, `filter.text == "edit"`, `search_edit.text`
unchanged, filter typing left the tree filtered, `search.pending` unchanged
until typing); `switching_tabs_shows_the_stored_results_without_a_new_query`
(`open`, results set for `main`, pick Files → `tree_rows > 0`, pick Text →
`tree_rows == 0`, `pending.is_none()`, `query == "main"`);
`a_short_query_keeps_the_pane_with_the_hint` (pick Text, type `a` → `open` true,
`tree_rows == 0`, the `Type at least 2 characters` text mounted);
`a_valid_query_reads_searching_before_the_reply` (pick Text, type `main` → the
`Searching…` text mounted and `Type at least 2 characters` absent, before any
`poll` call);
`picking_the_same_tab_keeps_an_error_without_re_issuing` (`open`, `error` set,
`results`/`pending` `None`, text `main` → pick `Text` → `pending` stays `None`
and the error text stays mounted);
`the_context_tab_sets_the_mode_and_status` (pick Context → `mode == Concept`,
`Search by context` mounted, each `IndexStatus` renders its line, `Idle` removes
it); `the_actions_menu_offers_the_four_items_and_new_file_prompts` (press `File
actions` → the four items mount; choose `New File…` →
`sidebar_input` is `NewEntry` and `actions_open` false);
`the_header_stays_inside_the_aside` (with the section at 760×700 and the aside
forced to 280 and then 180 via `pane_layout`, every element of the header has
`visible[0] + visible[2] <= aside right edge + 0.5` and the input's visible width
is at least 90 on the Files tab). Then `cargo test -p goko-rune` passes with
`assert_no_horizontal_overflow(&rt, 760.0)` still green in
`search_open_shows_results_in_aside_without_overflow`.

Progress: done
Result: `files_view` mounts `aside_header` above the tree or results; the strip, one input, and the `File actions` menu are in place; `toolbar`, `search_row`, and `mode_switch` are deleted; the header rows stretch (no `w_full` plus padding) so the strip and input stay inside the aside at 480, 280, and 180 points. `cargo test -p goko-rune` passed (978 tests), including `the_aside_header_mounts_the_tab_strip_and_one_input`, `picking_text_swaps_the_input_and_keeps_both_texts`, `switching_tabs_shows_the_stored_results_without_a_new_query`, `a_short_query_keeps_the_pane_with_the_hint`, `a_valid_query_reads_searching_before_the_reply`, `picking_the_same_tab_keeps_an_error_without_re_issuing`, `the_context_tab_sets_the_mode_and_status`, `the_actions_menu_offers_the_four_items_and_new_file_prompts`, `the_header_stays_inside_the_aside`, and `search_open_shows_results_in_aside_without_overflow`.

### Task 3 — Wiki, architecture, and changelog

Files: `docs/llm/domains/rune-editor.md` (the `RNE-concept-search` and
`RNE-content-search` blocks), `ARCHITECTURE.md` (the Context search bullet),
`CHANGELOG.md` (`## [Unreleased]`).

Interfaces: Consumes the shipped behavior of Tasks 1 and 2 and `delta.md`;
produces no code interface.

Change: Merge `delta.md`'s two `Replacement` blocks into
`docs/llm/domains/rune-editor.md` with `document-wiki`'s merge step, preserving
every other requirement on the page. In `ARCHITECTURE.md`, replace the sentence
that says the Files toolbar "always shows a search box, under the file filter"
with the tab strip and one input in the aside, keeping the `Text | Context` mode
names, the index location, and the sync/pass rules it states today. Add one
`### Changed` bullet under `## [Unreleased]` in `CHANGELOG.md` written for users:
the file filter and the code search now share one input under `Files | Text |
Context` tabs above the file tree, so the editor keeps its full height and the
file actions moved into the `File actions` menu.

Verify: `rg -n "search box is always available|under the file filter|always
shows two inputs|search row shows" docs/llm/domains/rune-editor.md
ARCHITECTURE.md` returns nothing; `rg -n "Files \\| Text \\| Context"
docs/llm/domains/rune-editor.md ARCHITECTURE.md CHANGELOG.md` finds the new
wording in all three; the two wiki blocks equal `delta.md`'s replacements
(modulo surrounding blank lines) and every other `### RNE-` heading count on the
page is unchanged; `git diff --stat docs/llm` touches only
`docs/llm/domains/rune-editor.md`.

Progress: done
Result: the two wiki blocks equal `delta.md`'s replacements; the stale wording is gone (`grep` for `search box is always available`, `under the file filter`, `always shows two inputs`, `search row shows` returns nothing); `Files | Text | Context` appears in the wiki, `ARCHITECTURE.md`, and `CHANGELOG.md`; `git diff --stat docs/llm` touches only `docs/llm/domains/rune-editor.md`; the page still has 33 `### RNE-` headings.

## Files inspected, no change

- `apps/rune/src/editor/pane_layout.rs` — aside width range, default, and grip
  used by the header-width checks.
- `apps/rune/src/editor/navigation.rs` — the go-to-definition fallback already
  forces Content and sets `open = true`, which lands on the `Text` tab.
- `crates/rune-kit/src/segmented.rs` — radiogroup, `focus_group`, Fill/Fit widths,
  and middle truncation reused for the strip.
- `crates/rune-kit/src/text_input.rs` — fill/min width and
  placeholder-as-accessible-name behavior reused by the one input.
- `crates/rune-kit/src/menu.rs` — anchored menus, item icons, `on_close`.
- `apps/rune/src/editor/changes.rs` — the anchored-menu pattern mirrored by
  `actions_menu_layer`.
- `apps/rune/src/main.rs:1915` — `assert_no_horizontal_overflow`, reused by the
  header-width checks.

## Approval Gate

Status: approved

## Decision Log

- 2026-11-19 — Approved: the user asked to implement (`/implement-task`) after
  `design.md`, `tasks.md`, and `delta.md` were presented, so the gate covers all
  three files.
- 2026-11-19 — The user chose PA2 from the chat analysis: a three-segment
  `Files | Text | Context` strip with one input, and the four file actions in an
  overflow menu, over PA1 (two tabs plus a separate `Text | Context` switch) and
  PA3 (keeping a full-width actions row).

