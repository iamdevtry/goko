# Delta

## MODIFIED Requirements

### RNE-concept-search

Previously: the search box sat on its own always-visible row under the file filter, switched between Text and Context by a `Search mode` radio group, and a cleared or shortened query returned the aside to the file tree.

Baseline:

````md
### RNE-concept-search
The system SHALL, when the Files toolbar's search box is in Context mode, rank text chunks of the checkout's tracked and non-ignored untracked files by how well they match the query's words, matching identifier words across `snake_case` and `camelCase`, and open a selected result at its matching line through the unsaved-draft guard, while Text mode remains a literal search.
#### Scenario: The search box is always available
- GIVEN the Files section is open
- WHEN the user has pressed no shortcut
- THEN the search box and its `Text | Context` switch are visible under the file filter with Text selected, and typing two characters searches
#### Scenario: Words match identifiers
- GIVEN Context mode is active and one chunk defines `reconnect_backoff_ms` while another only mentions "backoff" in a comment
- WHEN the user searches `reconnect backoff`
- THEN the chunk that defines the identifier ranks above the comment-only chunk
#### Scenario: The chosen mode is kept
- GIVEN the user switched the search box to Context
- WHEN the user changes checkout or searches again in the same session
- THEN the box stays in Context
#### Scenario: Filter and search are independent
- GIVEN search results are showing
- WHEN the user edits the file filter
- THEN the file tree shows, filtered by name, the search text and mode are kept, and editing the search box shows the results again
#### Scenario: The index builds in the background
- GIVEN a checkout that has no index
- WHEN Context mode is first used for it
- THEN the search row shows `Indexing done/total files`, answers from what is indexed so far, the editor stays responsive, and the unchanged query is issued again when indexing finishes
#### Scenario: The index follows the working tree
- GIVEN an index exists and a file is edited, added, deleted, or rewritten as a binary file
- WHEN the next refresh pass runs
- THEN results reflect the change, only the changed files are re-chunked, and a file that is no longer indexable no longer appears
#### Scenario: A just-edited file is found by the next query
- GIVEN a full pass has finished and a file is then added, edited, or deleted in the working tree
- WHEN the user searches in Context mode immediately, less than 10 seconds later
- THEN the first reply reflects the change and only that file was re-chunked; when the catch-up takes longer than one second the reply comes from the current index and is issued again when the catch-up finishes
#### Scenario: Ignored, binary, and outside files are not indexed
- GIVEN a git-ignored file, a binary file, a file over 1 MiB, and a symlink to a file outside the checkout each contain the query word
- WHEN the user searches in Context mode
- THEN none of them appears in the results
#### Scenario: Content entry points show Text mode
- GIVEN the search box is in Context mode
- WHEN the user presses `⇧⌘F`, or a go-to-definition finds no single declaration and shows the content search results
- THEN the box is in Text mode and shows the literal results
#### Scenario: Failure leaves Text mode intact
- GIVEN the index cannot be opened or rebuilt
- WHEN Context mode is used
- THEN the search row shows `Context search unavailable` with the reason and Text mode still searches literally
Evidence: `crates/app-core/src/code_index.rs`, `apps/rune/src/editor/search.rs`, `apps/rune/src/editor/files.rs`, `apps/rune/src/editor/backend.rs`, `apps/rune/src/editor/backend/apply.rs`, `apps/rune/src/editor/navigation.rs`, `apps/rune/src/editor/keyboard.rs`
````

Replacement:

````md
### RNE-concept-search

The system SHALL, when the Files aside's search pane is in Context mode, rank text chunks of the checkout's tracked and non-ignored untracked files by how well they match the query's words, matching identifier words across `snake_case` and `camelCase`, and open a selected result at its matching line through the unsaved-draft guard, while Text mode remains a literal search.

#### Scenario: The search pane is always available

- GIVEN the Files section is open
- WHEN the user has pressed no shortcut
- THEN the aside header shows the `Files | Text | Context` tabs with `Files` selected and one input under them, and picking `Text` or `Context` shows the input under that mode's placeholder

#### Scenario: Words match identifiers

- GIVEN Context mode is active and one chunk defines `reconnect_backoff_ms` while another only mentions "backoff" in a comment
- WHEN the user searches `reconnect backoff`
- THEN the chunk that defines the identifier ranks above the comment-only chunk

#### Scenario: The chosen mode is kept

- GIVEN the user picked the `Context` tab
- WHEN the user changes checkout or searches again in the same session
- THEN starting a search again still uses Context

#### Scenario: Filter and search are independent

- GIVEN search results are showing
- WHEN the user picks the `Files` tab and edits the file filter
- THEN the file tree shows, filtered by name, the search text and mode are kept, and picking `Text` or `Context` shows the results again

#### Scenario: The index builds in the background

- GIVEN a checkout that has no index
- WHEN Context mode is first used for it
- THEN the search pane shows `Indexing done/total files`, answers from what is indexed so far, the editor stays responsive, and the unchanged query is issued again when indexing finishes

#### Scenario: The index follows the working tree

- GIVEN an index exists and a file is edited, added, deleted, or rewritten as a binary file
- WHEN the next refresh pass runs
- THEN results reflect the change, only the changed files are re-chunked, and a file that is no longer indexable no longer appears

#### Scenario: A just-edited file is found by the next query

- GIVEN a full pass has finished and a file is then added, edited, or deleted in the working tree
- WHEN the user searches in Context mode immediately, less than 10 seconds later
- THEN the first reply reflects the change and only that file was re-chunked; when the catch-up takes longer than one second the reply comes from the current index and is issued again when the catch-up finishes

#### Scenario: Ignored, binary, and outside files are not indexed

- GIVEN a git-ignored file, a binary file, a file over 1 MiB, and a symlink to a file outside the checkout each contain the query word
- WHEN the user searches in Context mode
- THEN none of them appears in the results

#### Scenario: Content entry points show Text mode

- GIVEN the `Context` tab is showing
- WHEN the user presses `⇧⌘F`, or a go-to-definition finds no single declaration and shows the content search results
- THEN the `Text` tab is showing and displays the literal results

#### Scenario: Failure leaves Text mode intact

- GIVEN the index cannot be opened or rebuilt
- WHEN Context mode is used
- THEN the search pane shows `Context search unavailable` with the reason and Text mode still searches literally

Evidence: `crates/app-core/src/code_index.rs`, `apps/rune/src/editor/search.rs`, `apps/rune/src/editor/files.rs`, `apps/rune/src/editor/backend.rs`, `apps/rune/src/editor/backend/apply.rs`, `apps/rune/src/editor/navigation.rs`, `apps/rune/src/editor/keyboard.rs`
````

### RNE-content-search

Previously: the file filter and the search box were two always-visible inputs on two rows above the split, so both were mounted at once and neither was a tab.

Baseline:

````md
### RNE-content-search

The system SHALL keep a filename filter on the file tree and a search box, both always visible in the Files toolbar, and, in the search box's Text mode, search the checkout's file contents literally only after the trimmed query reaches two characters and 250 ms pass without typing, opening a selected result at its matching line through the unsaved-draft guard.

#### Scenario: Debounced content search

- GIVEN Text mode is active and the query has at least two trimmed characters
- WHEN typing pauses for 250 ms
- THEN exactly one checkout search is issued for the latest query; shorter queries never issue

#### Scenario: Open a match

- GIVEN content results are shown
- WHEN the user selects a result
- THEN the file opens at the matching line through the draft guard, or a same-file result reveals the line directly

#### Scenario: Grouped results

- GIVEN content results are shown
- WHEN the results pane renders
- THEN matches group under per-file header rows in first-seen order with indented `line: preview` rows, at most 500 displayed matches with a truncation note

Evidence: `apps/rune/src/editor/search.rs`, `apps/rune/src/editor/files.rs`, `apps/rune/src/editor/navigation.rs`, `crates/app-core/src/git_view.rs`
````

Replacement:

````md
### RNE-content-search

The system SHALL keep one input in the Files aside's header under a `Files | Text | Context` tab strip, always visible, with the `Files` tab filtering the file tree by name and the `Text` and `Context` tabs searching the checkout's file contents, and, in Text mode, search literally only after the trimmed query reaches two characters and 250 ms pass without typing, opening a selected result at its matching line through the unsaved-draft guard.

#### Scenario: One input, one tab at a time

- GIVEN the Files section is open
- WHEN the user picks `Files`, `Text`, or `Context`
- THEN the input shows that tab's text with the placeholder `Filter files`, `Search text`, or `Search by context`, and the text of the tab left behind is kept

#### Scenario: Debounced content search

- GIVEN Text mode is active and the query has at least two trimmed characters
- WHEN typing pauses for 250 ms
- THEN exactly one checkout search is issued for the latest query; shorter queries never issue

#### Scenario: Short query keeps the pane

- GIVEN the `Text` or `Context` tab is showing
- WHEN the input is cleared or holds one character
- THEN the pane shows `Type at least 2 characters` and no search is issued

#### Scenario: Open a match

- GIVEN content results are shown
- WHEN the user selects a result
- THEN the file opens at the matching line through the draft guard, or a same-file result reveals the line directly

#### Scenario: Grouped results

- GIVEN content results are shown
- WHEN the results pane renders
- THEN matches group under per-file header rows in first-seen order with indented `line: preview` rows, at most 500 displayed matches with a truncation note

Evidence: `apps/rune/src/editor/search.rs`, `apps/rune/src/editor/files.rs`, `apps/rune/src/editor/navigation.rs`, `crates/app-core/src/git_view.rs`
````
