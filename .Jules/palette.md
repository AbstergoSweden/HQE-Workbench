## 2026-02-04 - Settings Form Accessibility
**Learning:** Found that visual labels in SettingsScreen were not programmatic labels (missing `htmlFor`/`id`), causing accessibility issues and requiring test workarounds (`getByPlaceholderText`).
**Action:** Always check `htmlFor`/`id` pairing on forms first. When fixing, update tests to use `getByLabelText` to lock in the improvement. Use `selector: 'input'` with `getByLabelText` if labels contain interactive children to avoid ambiguity.
