# Todo

What is next, in order. The running log of the migration is [`../plans/host_migration.md`](../plans/host_migration.md).

1. **zwipe on a branch.** `zwipe-components` depends on orangui, keeps the Magic pieces, re-exports the rest. Gates and screenshots.
2. **portfolio on a branch.** Depend on orangui, copy the Zwipe architecture carousel into the portfolio. Gates and screenshots.
3. **cairn on a branch.** Depend on orangui with `toast`. Gates and a simulator build.
4. **Switch production** one host at a time, at a device, checking live after each.
5. **Re-pin scripts** move to `cargo update -p orangui`.

## Someday

- A theme-sync helper so a host checks its own theme list against orangui's in one line.
- A swatch page under `tests/` or `tools/` that renders all 62 theme blocks with their slots, for reviewing a new theme in one screenshot. zwipe has one; it could move here with the palettes.
- Rename `zwipe-components` once it is only the Magic pieces, if a better name turns up.
