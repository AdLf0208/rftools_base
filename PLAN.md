# Implementation plan — `rftoolsbase` slice 1

**Goal:** carry RFToolsBase's base machine family (`machine_frame`, `machine_base`,
`dimensionalshard`) and implement one working machine — the **Machine Infuser** — with
its real numbers (60,000 FE buffer, 600 FE receive/tick, one dimensional shard per
infusion, 5 ticks, cost 600 → 300 FE as the machine's own infused level climbs 0 → 256).

**Architecture:** a pure `runtime.rs` (cost curve, tick state machine, menu data) with
tests, a `support.rs` glue module (block entity, container, energy, tick, menu, tooltip,
placement, commands), one helper per stamped hook, and a content `.hmod` embedded in the
`.hnmod` (AE2 pattern) so one installed file gives block, items and behaviour.

**Tech stack:** Rust (pin `1d63e0a5`), horizon-mixins schema 6, content API 0.2,
Minecraft 26.3 assets.

**Spec:** `ports/_shared/queue/brief-rftoolsbase.md` + `FINDINGS.md` (jar inventory and
upstream behaviour).

## Global constraints

- Pin `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`, mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
- Only stamped hooks/helpers; API floor 307 (item-energy surface is not used; 142 is the
  block-energy surface, 65 the component registration).
- Builds/tests/e2e only through `ports/tools/build-slot.sh`, `CARGO_INCREMENTAL=0`,
  no `--target-dir`, no bare cargo.
- Packages: `content/rftoolsbase.hmod` + `rftoolsbase.hnmod` (port root), content also
  embedded at `mixins/rftoolsbase/data/rftoolsbase-content.hmod`.
- Licence MIT (rank 0), local-only until the maintainer approves publishing.

## Slice boundary (from the jar, not the name)

Carried: `machine_infuser` (block + item + menu), `machine_frame` (item),
`machine_base` (item), `dimensionalshard` (item + `rftoolsbase:shards` tag), their
recipes, advancements, loot table, models, textures, lang and a creative tab.

Deferred (recorded in `notes.md`): the three dimensional-shard ores and their worldgen
(worldgen-last ruling), `information_screen`, `security_manager`, and the tool items
(`smartwrench`, `smartwrench_select`, `filter_module`, `crafting_card`, `tablet`,
`tablet_filled`, `manual`, `network_monitor`, `teleport_probe`, `shard_wand`,
`orphaning_card`, `security_card`, `developers_delight`), plus `infused_diamond` /
`infused_enderpearl`.

## Tasks

### Task 1 — pure runtime (TDD)

**Files:** `mixins/rftoolsbase/runtime.rs`.

**Interfaces produced:** `CAPACITY`, `MAX_INSERT`, `BASE_COST`, `MAX_INFUSE`,
`INFUSING_TICKS`; `shard_cost(infused) -> i32`; `Machine { energy, infusing }`;
`Machine::tick(has_shard, target_infused: Option<i32>) -> Tick`;
`Tick { started, consumed_shard, finished, cost }`; `Machine::data_slots() -> [i32; 3]`;
`is_infusable_target(item) -> bool` is a support-side list, not here.

- [ ] Tests first (cost 0/1/255/256; no energy; exact 600; 5-tick countdown; 256 refuses;
      no shard; no target; data slots).
- [ ] Run `cargo test` through the slot runner and watch them fail (`todo!()`).
- [ ] Implement; `cargo test` green.

### Task 2 — glue and hooks

**Files:** `support.rs` + one helper per file + `horizon-mixin.toml`.

- [ ] `block_entity`, `block_entity_ticks`, `block_entity_container` (2 slots, menu),
      `block_entity_energy` (60,000 / 600, insert only), `block_entity_tick`,
      `screen`, `screen_draw`, `tooltip`, `place_block`, `component_types`,
      `register_commands`, `perform_command`, `embedded_content`.
- [ ] `/rftoolsbase info|energy|infused|held` for the harness readout.

### Task 3 — content

**Files:** `content/**` built by `tools/assemble_content.py`.

- [ ] Carried assets (models, textures, blockstate, lang), patched 26.3 item
      definitions, block/menu/item declarations, recipes (26.3 `#tag` form), the
      singular typed loot condition, `rftoolsbase:shards` + the `c:` tags the recipes
      name, creative tab.
- [ ] Build `content/rftoolsbase.hmod`; copy into `mixins/rftoolsbase/data/`.

### Task 4 — ladder and e2e

- [ ] fmt / clippy / test (crate), `stage → check → test` (embedded), `build`.
- [ ] `tools/e2e.sh` (EMBEDDED=1 + control leg): A empty machine; B insufficient FE does
      nothing; C exactly 600 FE infuses 1 shard → infused 1 and 0 FE; D at infused 255
      the cost is 301 FE; E at 256 nothing; F a non-shard does nothing; G an infused
      item placed as a block seeds the block's level (cost 482 at 100).
- [ ] `SELFTEST=1` exits 1; `check-e2e-logs.py --strict` green; `hmod-load.sh`.

### Task 5 — notes

- [ ] `notes.md` (class/tier, approach, licence+rank, slice boundary, hooks table,
      gaps, verification table, `Skills:` line), `FINDINGS.md`, `CONTENT.md`,
      `SOURCE.md`, `VALIDATION.md`.

## Decisions this plan makes

1. **The infuser block is infusable.** Upstream 1.20 registers it through
   `MachineInfuserTileEntity.createBlock()` (`.infusable()`); the 1.21 module builds a
   plain `BaseBlock` and loses the flag, leaving the machine with no in-pack target. The
   port restores the 1.20 intent, so the machine can infuse its own block item and the
   cost curve is observable.
2. **`facing` is horizontal (4 values).** The pin's only menued template is
   `minecraft:stonecutter`; upstream's `BaseBlock` faces all six ways. Up/down placement
   is a recorded deviation.
3. **The infused level is a registered item component** (`rftoolsbase:infused`, int),
   the pin's form of mcjtylib's `mcjtylib:infusable` component; the block entity stores
   the same int under `infused`, seeded from the item on placement.
4. **The energy buffer is API 142** (`energy` int in the compound), insert-only, as
   `GenericEnergyStorage(this, true, 60000, 600)`.
