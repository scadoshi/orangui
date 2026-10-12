# Todo

What is next, in order. The running log of the migration is [`../plans/host_migration.md`](../plans/host_migration.md).

1. **The README mark.** An ASCII orangutan at the top of the README; the line-art drafts are with the owner.
2. **cairn's simulator build** on its `orangui` branch, at a device.
3. **Switch production** one host at a time, at a device, checking live after each. The three branches are built and green; see the plan.
4. **Re-pin scripts** move to `cargo update -p orangui`.

crates.io is off the list for now; the plan's "Not now" section keeps what was learned.

## Someday

- A theme-sync helper so a host checks its own theme list against orangui's in one line.
- A swatch page under `tests/` or `tools/` that renders all 62 theme blocks with their slots, for reviewing a new theme in one screenshot. zwipe has one; it could move here with the palettes.
- Rename `zwipe-components` once it is only the Magic pieces, if a better name turns up.
