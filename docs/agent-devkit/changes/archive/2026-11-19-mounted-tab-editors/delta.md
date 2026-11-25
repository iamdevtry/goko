# Delta

## ADDED Requirements

### RNE-tab-switch-reuse

Page: `docs/llm/domains/rune-editor.md`

Search: checked every page with prefix RNE; `RNE-editor-tabs` covers the tab model and switching but not that each tab's editor surface stays mounted or that activation reuses its laid-out text, and no other requirement covers it.

Requirement:

````md
### RNE-tab-switch-reuse

The system SHALL show an open tab's editor immediately when that tab is activated — keeping one code surface mounted for every open tab while the tab is open, rendering only the active tab's surface visibly and interactively, moving focus to the document being activated, and reusing its laid-out text so activation lays out no document text again and issues no file read of its own.

#### Scenario: Switching between two large open files

- GIVEN two files of about 3 000 lines are open as tabs
- WHEN the user switches between their tabs
- THEN each file's text appears immediately with its own caret and scroll position, and no file read is issued for the activated tab

#### Scenario: Only the active tab answers input

- GIVEN two tabs are open and the first is active
- WHEN the user types in the code field
- THEN only the active document's buffer changes, the inactive document's buffer stays byte-for-byte unchanged, and the inactive surface is neither focusable nor pickable

#### Scenario: Closing a tab drops its surface

- GIVEN three tabs are open
- WHEN the user closes one
- THEN that tab's surface is gone and switching between the remaining tabs is still immediate

Evidence: `apps/rune/src/editor/files.rs`, `apps/rune/src/editor/code.rs`, `apps/rune/src/editor/state.rs`
````
