# Task lists

A GitHub-style task list mixes checked and unchecked items. Word has no reliable
native checkbox content control via docx-rs, so quilldown renders a checkbox glyph
as the item marker (no redundant bullet) and lines the text up like a normal list.

- [x] Wire up the parser
- [x] Emit native hyperlinks
- [ ] Ship a native checkbox content control
- [ ] Write the migration guide

## Inline formatting inside items

- [x] Support **bold**, _italic_, and `code` inside a checked item
- [ ] Support [links](https://example.com) inside an unchecked item

## Nested task lists

- [ ] Top-level task
  - [x] Nested done sub-task
  - [ ] Nested pending sub-task
- [x] Another top-level task

## Mixed with a plain bullet list

- Plain bullet, no checkbox
- [ ] Task in the same list
- Another plain bullet
