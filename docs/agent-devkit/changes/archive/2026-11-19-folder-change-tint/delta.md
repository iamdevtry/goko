# Delta

Approved: 2026-11-19 — folder rows take the most significant change tone beneath them

## MODIFIED Requirements

### RNE-change-marks

Previously: only changed files' names were tinted; folder rows stayed on the normal label colour, so a collapsed folder hid that it held changes.

Baseline:

````md
### RNE-change-marks

The system SHALL mark the current checkout's changed files in the Files section from the freshest Changes data it already holds — tinting a changed file's name in the file tree and showing a status dot on each open-file tab, conflicts included — using the shared status tones (modified and typechanged warning, added and untracked success, deleted and conflicted danger, renamed and copied accent), preferring an unstaged status over a staged one and a conflict over both, keeping Git-ignored rows dimmed, naming the status in each tab's accessible label, and issuing no Git read of its own.

#### Scenario: Changing files stand out in the tree

- GIVEN the checkout's Changes data lists `src/a.rs` as modified and `new.txt` as untracked
- WHEN the Files section renders the file tree
- THEN `a.rs` is drawn in the warning tone, `new.txt` in the success tone, and unchanged files and folder rows keep the normal label colour

#### Scenario: Conflict and unstaged statuses win

- GIVEN a path is both conflicted and listed as unstaged, and another path is staged as added and then edited again
- WHEN the marks are derived
- THEN the conflicted path uses the danger tone and names its conflict kind, and the re-edited path uses the warning tone of its unstaged status

#### Scenario: No dot without fresh data

- GIVEN an open tab whose path has no entry in the current checkout's Changes data, or the held data belongs to another checkout
- WHEN the tab strip renders
- THEN the tab shows no status dot and its accessible label is unchanged

#### Scenario: Unsaved edits keep their marker

- GIVEN a tab whose file is changed in Git and also has unsaved edits
- WHEN the tab strip renders
- THEN the tab shows the status dot in the status tone beside the existing unsaved marker, and its accessible label names the status and then "unsaved"

Evidence: `apps/rune/src/editor/files.rs`, `apps/rune/src/editor/changes.rs`
````

Replacement:

````md
### RNE-change-marks

The system SHALL mark the current checkout's changed files in the Files section from the freshest Changes data it already holds — tinting a changed file's name and every folder that holds one in the file tree, and showing a status dot on each open-file tab, conflicts included — using the shared status tones (modified and typechanged warning, added and untracked success, deleted and conflicted danger, renamed and copied accent), preferring an unstaged status over a staged one and a conflict over both for a file, and the most significant status beneath it for a folder (conflict, then deleted, modified or typechanged, added or untracked, renamed or copied), keeping Git-ignored rows dimmed, naming the status in each tab's accessible label, and issuing no Git read of its own.

#### Scenario: Changing files stand out in the tree

- GIVEN the checkout's Changes data lists `src/a.rs` as modified and `new.txt` as untracked
- WHEN the Files section renders the file tree
- THEN `a.rs` is drawn in the warning tone, `new.txt` in the success tone, and unchanged files keep the normal label colour

#### Scenario: A folder shows the change beneath it

- GIVEN `src/a.rs` is modified and `src/both.txt` is conflicted, and `new/` holds only an untracked file
- WHEN the tree renders with those folders collapsed
- THEN the `src` row uses the danger tone, the `new` row uses the success tone, and a folder with no changed file beneath it keeps the normal label colour

#### Scenario: Conflict and unstaged statuses win

- GIVEN a path is both conflicted and listed as unstaged, and another path is staged as added and then edited again
- WHEN the marks are derived
- THEN the conflicted path uses the danger tone and names its conflict kind, and the re-edited path uses the warning tone of its unstaged status

#### Scenario: No dot without fresh data

- GIVEN an open tab whose path has no entry in the current checkout's Changes data, or the held data belongs to another checkout
- WHEN the tab strip renders
- THEN the tab shows no status dot and its accessible label is unchanged

#### Scenario: Unsaved edits keep their marker

- GIVEN a tab whose file is changed in Git and also has unsaved edits
- WHEN the tab strip renders
- THEN the tab shows the status dot in the status tone beside the existing unsaved marker, and its accessible label names the status and then "unsaved"

Evidence: `apps/rune/src/editor/files.rs`, `apps/rune/src/editor/changes.rs`
````
