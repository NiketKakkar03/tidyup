# Rule Packs

The MVP uses a built-in deterministic extension-based rule pack.

Current built-in destinations:

- documents
  `txt`, `md`, `pdf`, `doc`, `docx`

- images
  `jpg`, `jpeg`, `png`, `gif`, `heic`

- spreadsheets
  `csv`, `xls`, `xlsx`

- archives
  `zip`, `tar`, `gz`

Rule-pack constraints:

- schema version must match
- destination directories must be safe relative names
- extensions cannot be ambiguous across rules

The current implementation does not yet load external rule-pack files, but the validation and rule identity model are already present in the core crate.

Example conceptual mapping:

```text
todo.md      -> Documents/todo.md
photo.jpg    -> Images/photo.jpg
budget.xlsx  -> Spreadsheets/budget.xlsx
archive.zip  -> Archives/archive.zip
```
