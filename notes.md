# RFToolsBase — port notes (slice 1: the machine family and the Machine Infuser)

Ported from **RFToolsBase 1.21-6.0.11** (`McJtyMods/RFToolsBase`, **MIT**), the
exact ATM10 NeoForge 1.21.1 jar `rftoolsbase-1.21-6.0.11.jar` (SHA-1
`9614bdc12d6c633801b254a983d30dd4cbeb5aee`, byte-identical to the pack's
`modlist.json` entry and Modrinth version `f8Tk2cfj`; provenance in
[`ports/_shared/carry-work/rftoolsbase/SOURCE.md`](../_shared/carry-work/rftoolsbase/SOURCE.md)).
Upstream behaviour was read from branch `1.21_neo`, commit
`4f7e15e64c7eb05365895773aa8f20f78ec4ac15` (2026-06-16, `version=1.21-6.0.11`)
and McJtyLib `1.21_neo` `ef393e608eebc188a6e1decbf110506a55cc0725` — analysis
only, this is a clean-room rewrite. What was read and the numbers extracted:
[`FINDINGS.md`](FINDINGS.md).

**Licence: MIT → rank 0 (permissive).** Local-only until the maintainer
approves publishing (ARCHITECTURE.md §8). Ported now because RFToolsBase is the
core of the RFTools family (five pack mods and XNet declare it required), and it
is the rank-0 `content`/`machines` core mod `precheck-mod.py rftoolsbase`
offers (`DISPATCHABLE`, jar licence MIT, no unported content dependency, no
worldgen in this slice).

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6) — the slice-1 build.
**Current format:** the flat **SDK package**, version **0.2.2**, mod API `0.2`, at
Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** / mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (re-pinned from `0b14f8a` on
2026-11-21, [`## Repin to the campaign pin`](#repin-to-the-campaign-pin-2026-11-21-0a875da));
the `.hnmod` container is retired (2026-11-18). The migration is
[`## Migration to the SDK format`](#migration-to-the-sdk-format-2026-11-18-sdk-0b14f8a);
the retired tree is evidence at
`ports/_shared/evidence/rftoolsbase/retired-hnmod-0.1.0/`.

Status: **ported-partial** — slice 1 (the base machine family, the dimensional
shard, and the Machine Infuser with its buffer, slots, infusion cycle, menu,
placement and tooltip) is implemented and was verified in-game at the campaign
pin in a **three-package game** (2026-11-21: the infuser scenario A–G and the two
recipes that name `rftoolsbase:machine_frame`). The gap list below is the rest of
the mod.

## Class + tier and the approach

- **Class:** `content`; **tier:** `machines` (`ports/ledger.csv`,
  `ports/_shared/source-map/atm10-sources.csv`).
- **Approach** (`horizon/docs/modpacks.md`, "What packs need and what runs
  today"): a **machines** shape — a block entity with a server tick, a declared
  energy capability, a container and a menu/screen — plus the carried items,
  assets, recipes and loot. Not a data-only block, and nothing here is `deep`.
- **Carry-over vs agent port:** the models, textures, blockstate, lang lines,
  loot table, recipes, advancements and the `rftoolsbase:shards` tag are carried
  from the jar (`tools/assemble_content.py`, decisions in
  [`CONTENT.md`](CONTENT.md)); the behaviour, the block entity, the container,
  the energy buffer, the tick, the menu, the screen, the tooltip, placement and
  the commands are the agent port.

## What the slice does

- **The machine.** `rftoolsbase:machine_infuser`, a `BaseBlock` on the
  **`minecraft:stonecutter`** template (vanilla's only menued template with a
  horizontal `facing`), standard-iron properties (hardness 2.0 / blast 2.0,
  correct tool, metal sound), placed facing the player, with the jar's own cube
  model and `block/infuser` front texture.
- **The buffer.** 60,000 FE (`infuserMaxRF`), declared through
  `block_entity_energy` (API 142, `Capabilities.Energy.BLOCK`): any side may
  fill it 600 FE a call (`infuserRFPerTick`), nothing may drain it
  (`GenericEnergyStorage(this, true, 60000, 600)` has `canExtract` false). The
  amount is the int `energy` in the block entity's compound.
- **The slots.** Slot 0 takes `rftoolsbase:shards` items (the dimensional shard,
  64 a stack); slot 1 takes one infusable machine (a stack limit of 1) and is
  also the output. Every face reaches both; automation may take from slot 1.
- **The infusion.** `tickServer`: while `infusing > 0` the countdown drops and
  the last tick raises the machine in slot 1 by one level; otherwise, if slot 0
  holds a shard **and** slot 1 holds an infusable machine below 256, `startInfusing`
  spends `(int)(600 * (2 - blockInfused/256) / 2)` FE and one shard and sets
  `infusing = 5`. **The cost comes from the block's own level**, not the item's
  (600 FE at 0, 301 at 255, 300 at 256); the item's level only decides whether
  it can still be raised.
- **The level on the item.** A registered item component
  `rftoolsbase:infused` (API 65), the pin's form of mcjtylib's
  `mcjtylib:infusable`: one int, 0..=256. Placing an infused machine item seeds
  the block's own level (`applyImplicitComponents` → `DefaultInfusable.load`),
  so the placed machine infuses more cheaply; the item's tooltip shows
  `Infused: N%`, as upstream's `BaseBlock.intAddInformation` does.
- **The menu.** `rftoolsbase:machine_infuser_menu` (2 slots, 4 data slots:
  `energy/32`, `capacity/32`, `infusing`, block `infused`), slots at upstream's
  (64, 24) and (118, 24) pixels in its 180 × 152 panel with the player
  inventory at (10, 70); the screen draws the FE line, the block's infused level
  and the countdown as text instead of the energy bar.
- **The family.** `rftoolsbase:machine_frame`, `rftoolsbase:machine_base` and
  `rftoolsbase:dimensionalshard` as items (1.21 registers frame/base as plain
  items, not blocks), with the jar's four recipes and advancements.
- **Harness controls.** `/rftoolsbase info <pos> [nonce]`, `/rftoolsbase energy
  <pos> <amount>`, `/rftoolsbase infused <pos> <amount>`, `/rftoolsbase level
  <pos> <amount>`, `/rftoolsbase held <amount>`. Upstream has no commands (its
  config and GUI are the controls); precedent `/charginggadgets info|energy`,
  `/powah info|set`. The nonce exists so the e2e reads its own reply, not an
  older one in the client's chat.

## Hooks (all stamped; no pending hook is named)

| Hook (API) | Helper | What it does |
|---|---|---|
| `component_types` (65) | `component_types.rs` | registers `rftoolsbase:infused` before content loads |
| `block_entity` (15) | `block_entity.rs` | the machine's block entity |
| `block_entity_ticks` (15) | `block_entity_ticks.rs` | ticks on the server |
| `block_entity_container` (16/17) | `block_entity_container.rs` | two slots, every face in, slot 1 out, the menu type |
| `block_entity_energy` (142) | `block_entity_energy.rs` | the 60,000 FE insert-only buffer |
| `block_entity_tick` (17) | `block_entity_tick.rs` | the five-tick infusion cycle |
| `screen` (19) | `screen.rs` | the menu's two slots and the panel size |
| `screen_draw` (19) | `screen_draw.rs` | FE, infused level and countdown readouts |
| `tooltip` (12) | `tooltip.rs` | the item's `Infused: N%` line |
| `place_block` (14) | `place_block.rs` | an infused item seeds the block it becomes |
| `register_commands` (8) | `register_commands.rs` | the five harness commands |
| `perform_command` (8) | `perform_command.rs` | their bodies |
| `content_root` (raw target) | `embedded_content.rs` | writes the bundle's own content out and loads it |

13 injections (12 named hook points + one raw `content_root` target), 2 modules,
**0 unstamped**.

## Pin facts that shaped this slice

1. **The cost is the block's, not the item's.** `startInfusing` reads
   `this.infusable.getInfusedFactor()` — the block entity's own level — while
   `isInfusable(outputStack)` reads the item's. The first e2e run mixed the two
   (it set the *item* to 255 and expected 301 FE); the runtime now takes both
   and a unit test pins the separation. This is the kind of error a
   differential number check catches and a reading does not.
2. **`native_component`/`native_set_component` (API 65) are the pin's per-item
   data.** The amount lives in the stack's components, survives save/load and
   travels in the inventory packet, so the infused level is a real component,
   not a name suffix.
3. **A block entity's `entity` binding inside `block_entity_tick` is the live
   one.** Writing `entity.nbt` there persists (the e2e reads it back), unlike
   the reverted write a nested `native_with_mod_block_entity` during a tick
   would do; the command path (outside the tick) and the tick binding agree.
4. **`native_server_item_in_tag` is the server-side tag read** (API 63), so the
   shard test is the real `rftoolsbase:shards` tag, not a hard-coded id; the
   content declares that tag.
5. **The infuser block is infusable in 1.20 and not in 1.21.** The 1.20
   registration used `MachineInfuserTileEntity.createBlock()` (`.infusable()`);
   the 1.21 module builds a plain `BaseBlock` and loses the flag, leaving
   RFToolsBase with no infusable block (the machine could only infuse sibling
   mods' machines). The port restores the 1.20 intent — the machine infuses its
   own block item — which is what makes the curve observable in this pack.

## Content (`.hmod`)

`content/` is generated from the jar by
[`tools/assemble_content.py`](tools/assemble_content.py); [`CONTENT.md`](CONTENT.md)
lists every carried and every written file and why. 42 entries: the machine
blockstate (4 facings), the machine/frame/base models, the two item models, six
textures, the 26.3 item definitions, `en_us.json`, the four recipes (26.3
ingredient form), the four advancements, the loot table (singular typed
`survives_explosion`), the `rftoolsbase:shards` tag, the block/menu/tab
declarations, four `c:` convention tags and the pickaxe tag.

## Embedded content (AE2 pattern)

The registry stores **one file per project** (playbook §1.0), so a port with both
halves could not be listed faithfully: an `.hnmod` is code-only and an `.hmod`
is content. This bundle carries its content, as the merged AE2 entry does:

- **`mixins/rftoolsbase/data/rftoolsbase-content.hmod`** is the built
  `content/rftoolsbase.hmod`, byte for byte (15,488 B, `f545dd79…6615`).
  `tools/package.py` packs `data/**` as `mixins/<id>/data/…`, with fixed zip
  timestamps, and refuses a copy that is not byte-identical to the built one.
- **`mixins/rftoolsbase/embedded_content.rs`** (`content_root` HEAD, bindings
  `root`/`enabled`/`dev`) compiles the file in with `native_data!` (API 41),
  writes it to `.horizon-cache/rftoolsbase-embedded-content.hmod` when the copy
  differs, appends it to the dev-mod list and returns its `content_root`; the
  second call stops at its own `dev.contains(&file)` guard (AE2's one-level
  recursion). An unwritable cache logs and `Continue`s instead of failing.

Verified in game with **only the bundle** (`EMBEDDED=1`, no `--dev-mod`): the
whole scenario A–G passes, and the file the helper wrote is byte-identical to
`content/rftoolsbase.hmod`. The control (`CONTROL=1`) launches the same binary
with `.horizon-cache` replaced by a plain file, so the helper refuses its
content: no machine item arrives and `setblock … rftoolsbase:machine_infuser`
leaves `minecraft:air` — the ids are unknown without the embedded file, so the
PASS is not vacuous.

### In-game transcript (quoted from `e2e-embedded.log`)

```
== control: the same game with no embedded content file ==
  without the embedded file: no machine item, the block stayed minecraft:air
  embedded content: 15488 B at /tmp/opencode/rftoolsbase-home.Kz4QfQ/.horizon-cache/rftoolsbase-embedded-content.hmod, identical to content/rftoolsbase.hmod
== C: exactly 600 FE infuses one shard, so the item goes 0 -> 1 ==
  Machine Infuser at [202, 101, 0]: 0/60000 FE, infusing 0/5, block infused 0/256, shard empty, target rftoolsbase:machine_infuser x1 infused 1, t=353 nonce=4
== D: at block level 255 a shard costs 301 FE ==
  Machine Infuser at [202, 101, 0]: 299/60000 FE, infusing 0/5, block infused 255/256, shard empty, target rftoolsbase:machine_infuser x1 infused 1, t=460 nonce=5
== G: an item infused to 100 seeds the block; a shard then costs 482 FE ==
  Machine Infuser at [202, 101, 8]: 118/60000 FE, infusing 0/5, block infused 100/256, shard empty, target rftoolsbase:machine_infuser x1 infused 1, t=835 nonce=9
PASS: … one shard costs 301 FE (600->299) … at item level 256 a shard is refused … an item infused to 100 placed as a block makes a shard cost 482 FE (600->118, 0->1)
EXIT:0
```

This makes the `.hnmod` the single listable artifact of this port; it does not by
itself publish it (licence approval is still required, ARCHITECTURE.md §8).

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Review fixes (2026-11-18)

The reviewer's source review of PR #1204 (`var/review-all/1204-rftoolsbase.md`) left one finding;
it is fixed in **0.2.1** (`40048 B`, sha256 `965003c8970f103385f2c8303fbfc72d0f0a3663beddc91e16c8535d1b9429de`).

- **`src/lib.rs:311` — the level-0 `/rftoolsbase info` command accepted arbitrary coordinates and
  revealed both slots' item ids and counts and the target's infusion level with no reach or
  inventory access.** The readout is now gated on the ordinary interaction rules: an operator (or
  the console) reads any machine, a player only one within `runtime::READ_REACH` (6 blocks, the
  vanilla interaction reach plus slack), and any other non-operator source nothing. Test:
  `runtime::tests::only_an_operator_or_a_nearby_player_reads_a_machine`.

Two harness facts were found while making the in-game leg green (both are scenario fixes, not
assertion changes): `/rftoolsbase held` wrote through `ItemPlace::PlayerHand` and answered success,
so the level is now written to the selected inventory slot and read back
(`player.set_inventory_slot`), and the placement step selected `--item`'s *first* machine stack in
the hotbar instead of the infused one (`ctl place` now uses the held, infused stack, in survival so
the placement consumes it).

**In-game leg:** `ports/tools/launcher-game.sh run rftoolsbase` → `PASS` + `E2E EXIT:0`, control
`FAIL: the machine item did not register` + `EXIT:1`, recorded in
`ports/_shared/evidence/rftoolsbase-e2e/`.

## Verification

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6); checkouts
`ports/_shared/pins/horizon-1d63e0a` and `.../horizon-mixins-5a64af9`. Every
build/test/e2e ran through `ports/tools/build-slot.sh` with
`CARGO_INCREMENTAL=0`; no bare cargo, no `--target-dir`. Evidence:
`ports/_shared/evidence/rftoolsbase/`.

| Step | Evidence | Result |
|---|---|---|
| runtime TDD (red) | `test-red.log` | **0 passed, 14 failed** — every body `todo!()` |
| runtime tests (green) | `test-green.log` | **14 passed**, 0 failed |
| fmt | `fmt.log`, `fmt-helper.log` | `cargo fmt --all -- --check` and `rustfmt --check mixins/rftoolsbase/*.rs` both exit 0 (empty output) |
| clippy | `clippy.log` | `--all-targets --locked --offline -- -D warnings`, exit 0, `Finished dev profile` |
| stage | `stage.log` | `stage_changed: true`, 2 modules, 13 injections |
| check | `horizon-check.log` | `cargo_exit: 0`, **0 warnings** |
| embedded test | `horizon-test.log` | `test result: ok. **14 passed**; 0 failed; 1240 filtered out` |
| fast build | `horizon-build.log` | `cargo_exit: 0`, 0 warnings |
| e2e | `e2e.log` | **PASS**, `EXIT:0` |
| fail-capability | `e2e-selftest.log` | `SELFTEST=1` → **EXIT:1** (`the machine holds 0 FE, expected 600`) |
| e2e, one file only (`EMBEDDED=1`, no `--dev-mod`) | `e2e-embedded.log` | **PASS**, `EXIT:0`; the control leg (same binary, `.horizon-cache` unusable) shows the ids unknown |
| embed fail-capability | `e2e-embedded-selftest.log` | `SELFTEST=1 EMBEDDED=1` → **EXIT:1** |
| log checker | `check-e2e-logs.py rftoolsbase --strict` | `logs=4 ok=4 missing-exit=0 bad-exit=0` |
| `.hmod` build/check | `package-hmod.log` | 15,488 B, `f545dd79…6615`, 42 entries, “no code: content only” |
| `.hnmod` (content embedded) | `package-hnmod.log` | 24,746 B, `fa10a947…f6c8`, 18 entries incl. `mixins/rftoolsbase/data/rftoolsbase-content.hmod` |
| `cargo horizon-mod check --kind native-mod` | `package-hnmod.log` | exit 0, `native-mod … for 26.3` |
| load-check | `hmod-load.log` | **PASS** with `--dev-mod <the .hmod>` on the shared unmodded game; the same ids unknown without it (`block: minecraft:air`) |

Both packages were rebuilt a second time and reproduced their SHA-256 exactly
(15,488 B / `f545dd79…6615`; 24,746 B / `fa10a947…f6c8`).

### In-game transcript (quoted from `e2e.log`)

```
== A: a machine placed from its item is empty ==
  Machine Infuser at [202, 101, 0]: 0/60000 FE, infusing 0/5, block infused 0/256, shard empty, target empty, t=181 nonce=1
== B: 599 FE is one short of the 600 a shard costs ==
  Machine Infuser at [202, 101, 0]: 599/60000 FE, infusing 0/5, block infused 0/256, shard rftoolsbase:dimensionalshard x1, target rftoolsbase:machine_infuser x1 infused 0, t=283 nonce=3
== C: exactly 600 FE infuses one shard, so the item goes 0 -> 1 ==
  Machine Infuser at [202, 101, 0]: 0/60000 FE, infusing 0/5, block infused 0/256, shard empty, target rftoolsbase:machine_infuser x1 infused 1, t=353 nonce=4
== D: at block level 255 a shard costs 301 FE ==
  Machine Infuser at [202, 101, 0]: 299/60000 FE, infusing 0/5, block infused 255/256, shard empty, target rftoolsbase:machine_infuser x1 infused 1, t=460 nonce=5
== E: at item level 256 a shard and 600 FE do nothing ==
  Machine Infuser at [202, 101, 0]: 600/60000 FE, infusing 0/5, block infused 0/256, shard rftoolsbase:dimensionalshard x1, target rftoolsbase:machine_infuser x1 infused 256, t=567 nonce=6
== F: a non-shard in the shard slot does nothing ==
  Machine Infuser at [202, 101, -4]: 600/60000 FE, infusing 0/5, block infused 0/256, shard minecraft:dirt x1, target rftoolsbase:machine_infuser x1 infused 0, t=686 nonce=7
== G: an item infused to 100 seeds the block; a shard then costs 482 FE ==
  Machine Infuser at [202, 101, 8]: 0/60000 FE, infusing 0/5, block infused 100/256, shard empty, target empty, t=749 nonce=8
  Machine Infuser at [202, 101, 8]: 118/60000 FE, infusing 0/5, block infused 100/256, shard empty, target rftoolsbase:machine_infuser x1 infused 1, t=835 nonce=9
PASS: a fresh machine is empty; 599 FE is one short of a shard's 600; exactly 600 infuses one shard and raises the item 0->1; at block level 255 a shard costs 301 FE (600->299) while the item still goes 0->1; at item level 256 a shard is refused; a non-shard is refused; and an item infused to 100 placed as a block makes a shard cost 482 FE (600->118, 0->1)
EXIT:0
```

The run is fail-capable three ways: `SELFTEST=1` flips C's assertion and exits
1; B, E and F are controls that must not change a number (599 FE, an item at
256, a non-shard); and the embedded control shows the ids unknown without the
package. G is the differential: the same 600 FE infuses a level-0 block for 600
and a level-100 block for 482, and the item's own level does not move the price.

### Packaged artifacts

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/rftoolsbase.hmod` | 15,488 B | `f545dd79ec631d471272548fc027d59b4c00b11bdc0ade65058f12cfabaa6615` | 42 |
| `rftoolsbase.hnmod` | 24,746 B | `fa10a94738c65e787a55de1eece435c74d3ee90014f01f8d32b66f28857ff6c8` | 18 |

The `.hnmod` carries the content mod as
`mixins/rftoolsbase/data/rftoolsbase-content.hmod` (15,488 B, the same SHA-256),
so it is the single artifact a registry entry needs.

### Campaign gates at this touch

- `python3 ports/tools/precheck-mod.py rftoolsbase` → exit 0, `DISPATCHABLE`,
  jar licence MIT (rank 0).
- `python3 ports/tools/check-e2e-logs.py rftoolsbase --strict` → 4/4.
- `python3 ports/tools/upstream_ready.py rftoolsbase` → blocked on the ledger
  status (`pending`, the orchestrator records `ported-partial`) and this
  `Skills:` line, which is present; nothing about the artifact, the pin or the
  logs is a blocker.
- `tools/test_template.py` and `tools/test_stage.py` are the inherited scaffold
  fixtures; they do not apply to this bundle (the template test needs
  `native-hooks.json` in a no-catalog workspace and the stage test drives the
  old `ports/horizon` layout). They are kept unmodified so the scaffold stays
  comparable with `ports/_template`; the real ladder above is the evidence.

## Deviations and gaps (slice 1)

1. **The infuser's block is infusable.** Its 1.21 registration drops upstream's
   `.infusable()` flag (1.20's `createBlock()` sets it). Restored on purpose, so
   the machine has a target and the cost curve is observable; the item's own
   level only decides whether it can be raised (as upstream).
2. **`facing` is horizontal (4 values).** `minecraft:stonecutter`, the pin's
   only menued template, has four; upstream's `BaseBlock` faces all six ways, so
   placing a machine facing up/down is lost (the blockstate drops those two
   variants). Recorded rather than faked.
3. **`rftoolsbase:infused` is a component, not `mcjtylib:infusable`.** The
   library's component namespace does not exist in this tree; the port's own
   component carries the same int and is read by the same rules. Other RFTools
   mods' machines are not in this port, so only this machine's items carry it.
4. **A broken machine loses its FE and its slots.** Upstream's loot table copies
   mcjtylib's `item_energy`/`item_inventory` components onto the drop; the pin
   has no loot-function hook for a mod block's drop, so the drop is the plain
   block item and the two slots spill through the engine's container handling.
5. **The block's level is stored in the block entity, not a component.** As
   upstream (`DefaultInfusable.save/load`), so breaking the machine also loses
   its infused level; placing an infused item seeds it.
6. **No GUI texture or bars.** The carried `textures/gui/infuser.png` is not
   drawn: the menu uses the pin's plain panel and slot frames, and the energy bar
   becomes the `FE` and `infused` text lines. No hover tooltip inside the menu.
7. **No per-slot insert rules.** Upstream refuses a non-shard in slot 0 and a
   non-infusable item in slot 1 at `insertItem`; here both slots accept any
   item and the **tick** refuses what it cannot use (a non-shard sits in slot 0
   doing nothing, a machine at 256 sits in slot 1 doing nothing). The
   menu-slot-rules hook cannot reach a bundle.
8. **A stack of more than one in slot 1 is treated as one machine item**
   (upstream limits slot 1 to one). `ctl container`/the menu put one there; a
   hopper could put more and only the first is raised, one level per five ticks.
9. **No config file.** The four numbers are constants (the upstream defaults
   256 / 600 / 60,000 / 600); `config/` is not created.
10. **No particles, sounds, `lit`-style visuals or comparator output**, and no
    hopper automation test (the faces are declared, but only `ctl container`
    drove them).
11. **Block-to-block FE transfer is not exercised**: this build has one energy
    block, so the `Capabilities.Energy.BLOCK` insert path is declared and set
    through `/rftoolsbase energy` but no neighbour filled it.
12. **Not carried:** the three dimensional-shard ores and their worldgen
    (worldgen-last), `information_screen`, `security_manager`, `infused_diamond`
    / `infused_enderpearl`, the tool items (`smartwrench`, `smartwrench_select`,
    `filter_module`, `crafting_card`, `tablet`, `tablet_filled`, `manual`) and
    the Patchouli guide (unmapped library).

## Provenance (upstream analysis)

Repository `https://github.com/McJtyMods/RFToolsBase`, branch `1.21_neo`, commit
**`4f7e15e64c7eb05365895773aa8f20f78ec4ac15`** (2026-06-16,
`version=1.21-6.0.11`); McJtyLib `1.21_neo`
`ef393e608eebc188a6e1decbf110506a55cc0725`. Files read and behaviour extracted:
[`FINDINGS.md`](FINDINGS.md); the jar stays as licence evidence in
`ports/_shared/licence-cache/rftoolsbase-1.21-6.0.11.jar`. Both shallow clones
were deleted after the findings were written. Clean-room rewrite: no Java was
copied or translated and no jar was decompiled.

## Review repairs (PR #597)

The reviewer's blocking item is staging (the pinned artifact absent from the
registry release/staging and HTTP 404), which is the orchestrator's
(horizon-mods#686). The additional source-review item — *"Neither the native
archive nor its nested content archive contains LICENSE/NOTICE … Include the
original notice in the artifact"* — is fixed: `mixins/rftoolsbase/LICENSE` and
`NOTICE.md` travel inside the `.hnmod`, and both sit at the embedded content
archive's root. The stale audit tooling the reviewer flagged (`tools/audit/src/main.rs`
rejecting the cancellable embedded-content injection, and the one-injection
counts in `test_stage.py`/`test_template.py`) is repaired the same way as
`elevatorid`'s.

### Verification after the repairs

- Artifact: `rftoolsbase.hnmod`, 27,289 B, SHA-256 `fdc79718d95218ad35e54d7da2af178354e52d3cbc499cb31fc6b9c83cfb1bcc` (rebuilt twice, identical bytes; `tools/package.py`; the embedded content equals `content/rftoolsbase.hmod`).
- `python3 ports/tools/upstream_ready.py rftoolsbase` and `check-skills.py --notes ports/rftoolsbase`: exit 0.

## Migration to the SDK format (2026-11-18, SDK `0b14f8a`)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) retired the `.hnmod` container (#17992), and
the registry closed this port's submission (PR **#823**) with *"republish this as a native `.hmod`
with the SDK from the next Nightly"*. The port is now **one flat package** that carries both halves.

- **Pin:** Horizon **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (nightly 2026-11-18), horizon-mixins
  **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, mod API **`0.2`**, game 26.3. The SDK is fetched
  into `.horizon/sdk/` by any `cargo horizon-mod` command and is never packed.
- **Version:** the port's own **0.2.0** (the retired bundle was 0.1.0; the old content half
  declared 0.1.0 too, and would have had to move anyway).
- **Class + tier unchanged:** class **content**, tier **machines** — the same block entity, FE
  buffer, two slots, menu, server tick and carried content, over the same pure runtime. No feature
  is added or removed by the migration.

### File map — created

| Path | What it holds |
|---|---|
| `horizon-mod.toml` | the only manifest and the only identity: `id = "rftoolsbase"`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native]` pinned to `0b14f8a` / `e59da90` |
| `Cargo.toml` | one crate; the only dependency is `horizon-mod-api` from `.horizon/sdk/` (no `rust-version`: the scaffold declares none) |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks` (`register`, `component_types`, `register_commands`, `run_mod_command`), the `MachinePart` (`BlockPart` + `BlockEntityPart`: ticks, tick, screen, screen_draw), the `MachineItem` (`ItemPart`: tooltip, on_block_placed_by), the NBT/command/transaction glue and 7 glue tests |
| `src/runtime.rs` | the retired `mixins/rftoolsbase/runtime.rs`, **byte-identical** (`cmp` clean, SHA-256 `e6ffec7f164f65f0c641cca183b9fae8b420ed139b80ab45d92b0dcea1064425`); its 14 tests ran green before any glue existed, and `cargo test runtime::` re-runs them alone (`14 passed; 7 filtered out`) |
| `assets/rftoolsbase/**` | the carried assets, `cmp`-clean from the retired `content/assets/` |
| `data/**` (`rftoolsbase/`, `c/`, `minecraft/`) | the carried data, `cmp`-clean from the retired `content/data/` |
| `rftoolsbase.hmod` | the delivered package, port root, 39,218 B, SHA-256 `f754cd0db35461e6617f65db031efdf9c6a85151c4238e8c6394cbc18204ac20`, 49 entries |
| `.gitignore` | `/target`, `/.horizon`, `/Cargo.lock` |

### File map — deleted (and where it went)

| Deleted | Why | Where it went |
|---|---|---|
| `mixins/rftoolsbase/` (`horizon-mixin.toml`, `runtime.rs`, `support.rs`, 13 per-hook helpers, `data/rftoolsbase-content.hmod`) | the container, the plan/injection schema and the per-hook helper files are not part of any SDK package; `runtime.rs` moved to `src/`; the hook bodies became two Parts, three registrar declarations and two `Hooks` methods | `ports/_shared/evidence/rftoolsbase/retired-hnmod-0.1.0/mixins/` |
| `rftoolsbase.hnmod` (27,289 B) | unloadable on launcher ≥ 0.2.70 | same folder |
| `content/` (its `horizon-mod.toml`, `rftoolsbase.hmod` 15,488 B, `target/`) | a second package; the flat package carries `assets/` and `data/` itself | `assets/` + `data/` at the package root; the rest to the evidence folder |
| `compatibility.json` | `[native]` in `horizon-mod.toml` is the pin | evidence folder |
| `rust-toolchain.toml` | not part of any new package; the pin decides the toolchain | evidence folder |
| `Cargo.toml`, `Cargo.lock` (the old runtime crate) | replaced by the one SDK crate; the old lock carried the retired workspace | evidence folder |
| `tools/horizon.py`, `package.py`, `package.sh`, `build.sh`, `ladder.sh`, `assemble_content.py`, `add-notices.py`, `test_stage.py`, `test_template.py`, `hmod-load.sh`, `audit/` | the staged/mixin build machinery, the `.hnmod` packer, the content assembler, the stampers and the content-only load check; none applies to a package that is code *and* content at once | evidence folder |

**Kept on purpose:** `tools/e2e.sh` (its assertions are format-independent; the header now says
it needs a launcher-built game and the two embedded-content legs are gone), `tools/parse_info.py`
(the e2e's line parser), `FINDINGS.md`, `CONTENT.md`, `PLAN.md`, `LICENSE`, `LICENSE-NOTICE.md`
(both travel inside the `.hmod`), `README.md`/`VALIDATION.md` (rewritten for the flat layout —
`README.md` is packed, so a stale one would ship), this file.

**The embedded content is gone.** The retired bundle wrote its content half to
`.horizon-cache/` from a `content_root` injection (the AE2 pattern) because the registry kept one
file per project. The flat SDK package carries `assets/` and `data/` **inside its own `.hmod`**, so
that injection, the `native_data!` blob, the cache write, its control leg and the second package are
all deleted — the migration's biggest simplification.

### Helper → hook table (the retired 13 injections, and where each lives now)

| Retired (`.hnmod`) | SDK 0.2 | Where |
|---|---|---|
| `hook = "component_types"` + `component_types.rs` (API 65) | **`Hooks::component_types`**: `event.types.push("rftoolsbase:infused")` | `impl Hooks for Mod` |
| `hook = "block_entity"` + `block_entity.rs` (API 15; returned `Some((id, Compound::new()))`) | **no hook**: `r.block_entity_type("machine_infuser").blocks(&["rftoolsbase:machine_infuser"]).build()` declares the entity, and a declared type starts with an empty compound — the same state the helper seeded | `impl Hooks::register` |
| `hook = "block_entity_ticks"` + `block_entity_ticks.rs` (API 15) | **`BlockEntityPart::block_entity_ticks`**, `event.ticks = true` | `impl BlockEntityPart for MachinePart` |
| `hook = "block_entity_container"` + `block_entity_container.rs` (API 16/17) | **`r.block_container(&block, ModContainer { … })`**: two slots, all faces reaching both, slot 1 out, mod menu | `impl Hooks::register` |
| `hook = "block_entity_energy"` + `block_entity_energy.rs` (API 142) | **`r.block_energy(&block, ModEnergyBuffer { … })`**: 60,000 FE, 600 a call, insert-only; the amount stays the `energy` int | `impl Hooks::register` |
| `hook = "block_entity_tick"` + `block_entity_tick.rs` (API 17) | **`BlockEntityPart::block_entity_tick`**: `event.entity.nbt/items/data`, `event.pos`; answers `Unchanged` when nothing moved | `impl BlockEntityPart for MachinePart` |
| `hook = "screen"` + `screen.rs` (API 19) | **`BlockEntityPart::screen`** returning `Screen { 180 × 152, slots (64,24) and (118,24), inventory (10,70) }` | `impl BlockEntityPart for MachinePart` |
| `hook = "screen_draw"` + `screen_draw.rs` (API 19) | **`BlockEntityPart::screen_draw`**: `event.draw.text(…)` at the same three positions | same |
| `hook = "tooltip"` + `tooltip.rs` (API 12) | **`ItemPart::tooltip`**, attached to the machine item; `event.lines.push((Text::literal(…), 0xFFFF_FF55))` | `impl ItemPart for MachineItem` |
| `hook = "place_block"` + `place_block.rs` (API 14) | **`ItemPart::on_block_placed_by`**: the item's block was placed; write `infused` into the block entity's compound | same |
| `hook = "register_commands"` + `register_commands.rs` (API 8) | **`Hooks::register_commands`** with `Command::new` / `Command::op`; the nonce is an optional `[nonce:word]` | `impl Hooks for Mod` |
| `hook = "perform_command"` + `perform_command.rs` (API 8, split the raw line) | **`Hooks::run_mod_command`** returning `Flow`: typed `BlockPos` / `Integer` arguments, no string parsing | same |
| `[[injection]] embedded-content` (`content_root` HEAD) | — (content ships in the package) | — |

### What the glue converts, and why each conversion is not a behaviour change

- **The shard test.** `game.native_server_item_in_tag(stack, "rftoolsbase:shards")` became
  `host.tags().key(&Identifier::parse("rftoolsbase:shards"))` + `Tags::is_in_tag(stack.item(), key)`
  — the same merged item tag, resolved by name each tick (the tag key is retained once resolved).
- **The infused component.** `game.native_component`/`native_set_component` became
  `stack.components().and_then(|p| p.as_nbt().get(…))` and `stack.apply_components(&patch)`; the
  component id is still registered in `component_types` before content loads, so the content's
  items load under the same id.
- **The item writes.** `native_with_mod_block_entity` (a write outside the tick) became a
  `host.transaction`: `BlockEntityRef::set_item_slot` for slot 1 (`/rftoolsbase infused`) and
  `ItemPlace::PlayerHand{…}.edit` for the held stack (`/rftoolsbase held`), both journaled.
- **The block's state writes.** `entity.nbt` in the tick is still the saved compound; the commands
  read `BlockEntityRef::data` and write it back with `replace_data`. The retired
  `game.world.modified.insert(…)` (`setChanged`) has no equivalent and is not needed: the tick
  answers `Changed` only when it did something, which is the engine's own dirtying rule.
- **`native_feedback`** became `host.commands().feedback(Text::literal(…))`, which honours the
  source's silence and a swapped-in player, as before.
- **`native_text`** became `ScreenDrawEvent::draw.text(…)`; positions and the grey are unchanged.
- **The block's own level** is still the `infused` int in the block entity's compound, seeded on
  placement from the item's `rftoolsbase:infused` component. The retired `block` id test in
  `on_place` is unnecessary: the Part is attached to the machine item, so the hook hears only it.

### Build and check (verbatim)

`build-slot.sh` speaks the retired tool, so `cargo horizon-mod` runs **directly in the port**, with
`BUILD_LOCAL=1` (the second machine has no such path and `cargo horizon-mod` needs the port's own
`.horizon/sdk/`) and `CARGO_INCREMENTAL=0`; only the compile steps went through a slot.

```sh
# 0. the SDK must be fetched before anything compiles (`cargo test` alone fails on a missing
#    `.horizon/sdk/api/crates/horizon-mod-api/Cargo.toml`)
cd ports/rftoolsbase && cargo horizon-mod build
# 1. the pure runtime, before any glue existed
CARGO_INCREMENTAL=0 cargo test                       # → 14 passed
# 2. glue + all tests, lints, formatting
CARGO_INCREMENTAL=0 cargo test                       # → 21 passed (14 + 7)
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings   # → 0 warnings
cargo fmt --check                                    # → clean (never --all)
# 3. package, then the registry's own checks on the delivered file
rm -f Cargo.lock
CARGO_INCREMENTAL=0 cargo horizon-mod build           # → Built ./target/horizon-mod/rftoolsbase.hmod
cp target/horizon-mod/rftoolsbase.hmod ./rftoolsbase.hmod
cargo horizon-mod check rftoolsbase.hmod              # → CHECK_EXIT:0
cargo horizon-mod fits rftoolsbase.hmod --build nightly  # → "0b14f8a  made for", FITS_EXIT:0
sha256sum rftoolsbase.hmod
```

Measured (evidence `ports/_shared/evidence/rftoolsbase/sdk-0b14f8a/`): **21 passed**; clippy 0
warnings; `fmt --check` clean; package **39,218 B**, SHA-256
`f754cd0db35461e6617f65db031efdf9c6a85151c4238e8c6394cbc18204ac20`, **49 entries**; rebuilt a second
time from the same sources (`Cargo.lock` removed and `.horizon/sdk/` left in place) → the **same
bytes**; `check` and `fits` both exit 0:

```
rftoolsbase 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  39218 bytes, SHA-256 f754cd0db35461e6617f65db031efdf9c6a85151c4238e8c6394cbc18204ac20, unsigned
RFToolsBase:
0b14f8a  made for
```

The package's `uses.json` (354 entries) names the surface the glue actually uses — the Parts'
hooks `hook:block_entity_ticks`, `hook:block_entity_tick`, `hook:screen`, `hook:screen_draw`,
`hook:tooltip`, `hook:on_block_placed_by`, the global `hook:component_types`,
`hook:register_commands`, `hook:run_mod_command`; the registrar calls
`method:DefaultCapabilitiesRegistrar::block_energy` / `::block_container`,
`handle:Registrar::block_entity_part`, `handle:BlockEntityTypeBuilder::blocks` / `::build`; and
the handles
`BlockEntityRef::data` / `::replace_data` / `::item_slot` / `::set_item_slot`,
`Tags::is_in_tag`, `ItemPlace::edit`, `LevelRef::game_time`, plus the content half's
`hook:blocks` / `hook:items` / `hook:command` (the carried `horizon/block`, `horizon/item` and
`horizon/menu` declarations).

### Traps this migration cost

1. **Delete the old layout before the first build.** The project root may hold only package paths;
   `mixins/`, the `.hnmod` and `content/` moved to evidence first.
2. **`cargo test` cannot fetch the SDK.** `.horizon/sdk/api/crates/horizon-mod-api` does not exist
   until one `cargo horizon-mod build` (or `new`/`hooks`) has run in the port.
3. **`build-slot.sh run` forwards to the second machine, which has no such path.** `BUILD_LOCAL=1`
   keeps the step local, and the runner does **not** `cd` into the port: wrap every command.
4. **`cargo fmt --all --check` reads the fetched SDK** (a path dependency) and reports its
   `build.rs` diffs. Use plain `cargo fmt` / `cargo fmt --check`; it leaves `runtime.rs` alone
   (`cmp` confirmed after every glue edit).
5. **`Cargo.lock` is packed and changes the artifact.** Any `cargo test`/`clippy` writes one;
   `rm -f Cargo.lock` right before the build, and `/Cargo.lock` in `.gitignore`.
6. **A 38-char hash reads as missing pins** (`native requires game and exact revisions`). Both
   commits are 40 hex, and the mixins commit is `e59da9094f0c…`, **not** the retired campaign pin
   `5a64af93…`.
7. **`cargo horizon-mod check` with no file checks wasm**, not the package (`can't find crate for
   'core'`). Always `check <file.hmod>`; `cargo check`/`test` are for the native build.
8. **`capabilities = ["registries"]` is required** even where it reads as a claim; the schema has
   exactly that value. Here it is also true (a block entity type, a menu and a component).
9. **`BlockEntityPart: BlockPart`** — one type answers both, so an empty `impl BlockPart for
   MachinePart {}` is needed beside the `BlockEntityPart` impl, and both need
   `#[mod_api::horizon_part]`.
10. **`host.levels()` is a `ServerHost` service, not a `Host` one** (`E0599: no method named
    levels`). `placed_level` therefore takes `&dyn ServerHost`; the `Host`-only reads stay
    `Host`.
11. **`Registrar::item_part` takes the `ItemKey` by value**, not by reference (`E0308` in the
    first draft).
12. **Handle writes are `&mut dyn ServerHost` methods, reads are `&dyn Host` ones.** Inside
    `host.transaction(…)`, `BlockEntityRef::set_item_slot` returns `Result<bool, _>`, so the
    transaction yields `bool` directly — not a nested `Result` (the first draft assumed nesting).
13. **Do not pass a borrowed read into `feedback(host, …)`** (`E0502`: `*host` is borrowed both
    mutably and immutably). Compute the line into a `String` first.
14. **Clippy `too_many_arguments`** on the eight-argument info formatter; the three state numbers
    are now one tuple.
15. **A `Vec` array literal is not `Copy`** in a test (`E0277`); use `array::from_fn` for the six
    faces.
16. **`BlockEntityView::id()` is the block entity *type* id**, and the Part dispatch already
    filters on it, so the tick needs no id test. The declared type happens to share the block's
    name (`rftoolsbase:machine_infuser`) because the machine is the family's only entity.
17. **The retired raw command parsing is gone.** `run_mod_command` gets typed arguments, so
    `parse_pos`/`parse_pos_amount` disappeared; the e2e's `parse_info.py` still parses the reply
    line, unchanged.
18. **The build packs root `README.md`.** Documentation travels inside the package, so a stale
    description of the retired layout would ship; `README.md` and `VALIDATION.md` were rewritten
    before the recorded build.
19. **The e2e could not be re-run here.** It needs a launcher-built game
    (`horizon-launcher native-build . --instance <i> --trust-native-code`) and `horizon-launcher`
    is not installed. `tools/e2e.sh` keeps its assertions and lost only the embedded-content legs.

### Gaps after the migration

1. **No in-game run at `0b14f8a`.** The shared binaries are at `1d63e0a5`/`41f822b6` and no
   launcher is installed, so the behaviour's last in-game proof is the 0.1.0 `.hnmod` run. What is
   proved here: the runtime is byte-identical and its 14 tests pass, the glue type-checks against
   the pinned API crate, and the package passes `check` and `fits`. **This is why the port stays
   `ported-partial`.**
2. **The menu is unverified at this pin.** `ModContainer::menu` carries the content's own
   `rftoolsbase:machine_infuser_menu` (declared in `data/rftoolsbase/horizon/menu/`,
   `slots = 2`, `data_slots = 4`); whether the game resolves a mod menu id for a mod block here is
   a gameplay fact the e2e must show (the retired `.hnmod` opened it).
3. **The placement seeding is unverified.** `on_block_placed_by` writes `infused` into the block
   entity's compound; if the entity did not exist yet at that moment, scenario G would fail (it
   would cost 600 FE instead of 482). It is the closest SDK position to
   `applyImplicitComponents`, and the retired `place_block` hook confirmed the timing there.
4. **The screen text colour is unverified.** `DrawContext::text` takes `0xAARRGGBB` and the port
   passes `0xFF40_4040`; the retired API took `0x404040` through `native_text`. The positions and
   strings are unchanged, but only an offscreen screenshot can prove the colour.
5. **The two command item writes are unverified in game** (`set_item_slot` in a transaction, and
   `ItemPlace::edit` for the held stack). The retired path used `native_with_mod_block_entity` and
   the same stack edits.
6. **Everything the slice did not implement stays a gap** (the three shard ores and their
   worldgen, `information_screen`, `security_manager`, `infused_diamond`/`infused_enderpearl`, the
   tool items and the Patchouli guide) — see the slice-1 gap list above, unchanged.

### Changelog for the resubmission

> Supersedes **#823**, closed because Horizon retired `.hnmod` (#17992). Changes: one flat SDK
> package with `api = "0.2"` and `[native]` pinned to `0b14f8a`/`e59da90`; the thirteen retired
> injections became one `impl mod_api::Hooks` (`register` declaring the machine's block entity type,
> its 60,000 FE domain and its two slots; `component_types` for `rftoolsbase:infused`;
> `register_commands`/`run_mod_command` for `/rftoolsbase info|energy|infused|level|held`), one
> `BlockEntityPart` (ticks, the infusion tick, the screen and its readouts) and one `ItemPart` (the
> `Infused: N%` tooltip and the level a placed item seeds); the carried block, items, menu, tab,
> models, textures, lang, recipes, loot, advancements and tags now ship **inside the same
> `.hmod`** (`assets/`, `data/`), so the AE2 embedded-content workaround and its control leg are
> gone; the item component is read and written with `ItemStack::components`/
> `apply_components`, the block's saved state with `BlockEntityRef::data`/`replace_data`/`set_item_slot`
> in a transaction, and the shard test with `Tags::is_in_tag`; `runtime.rs` and its 14 tests are
> unchanged. Version 0.2.0.

## Repin to the campaign pin (2026-11-21, `0a875da`)

### Why

`rftoolsstorage` 0.2.5 had to record **`blocked(rftoolsbase:machine_frame)`** for its own
`storage_scanner` recipe (`ToT` / `gFg` / `ToT`, `F = rftoolsbase:machine_frame`), and
`rftoolspower` 0.1.1's phase M proved the same block in `cTc` / `cFc` / `cTc` crafts nothing. Both
packages are at the campaign pin; this port still declared `0b14f8a`, and
`ports/tools/launcher-game.sh check_pins` **refuses a game whose packages disagree on
`[native] horizon`**. So no two-mod RFTools leg could exist, and no child's recipe that names the
frame could be crafted anywhere. Moving this port to the pin is what makes the family buildable as
one game.

### What changed

- `[native] horizon` → **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`**; mixins stay
  `e59da909…`; version **0.2.2** in `horizon-mod.toml` **and** `Cargo.toml` (the launcher reuses a
  native build whose manifest is unchanged, so the bump is what forces a fresh compile).
- **`.horizon/sdk` deleted** before the first build (HANDOVER §9d.17a: a repin alone leaves the old
  surface cached). `cargo horizon-mod build` answered
  `SDK 0a875da (build 0a875da91dbb0fdcb7ba8c42c11341aa229af51a) in ./.horizon/sdk`.
- **No source change.** `src/runtime.rs` and `src/lib.rs` are byte-identical to 0.2.1; `README.md`
  was updated (it is packed, so it must not describe the old pin).
- `tools/e2e-craft.sh` is new (the three-package craft differential); `tools/e2e.sh` now honours
  `E2E_TMP` for its scratch home and socket (it used `/tmp` unconditionally).

### Ladder (all in `build-slot.sh` leases, `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`)

| Step | Command | Evidence | Result |
|---|---|---|---|
| SDK | `rm -rf .horizon/sdk && cargo horizon-mod build` | `sdk-0a875da/sdk-build-1.log` | `SDK 0a875da (build 0a875da91…)` |
| tests | `cargo test` | `sdk-0a875da/sdk-test.log` | **22 passed**, 0 failed (14 runtime + 8 glue) |
| clippy | `cargo clippy --all-targets -- -D warnings` | `sdk-0a875da/sdk-clippy.log` | exit 0, **0 warnings** |
| fmt | `cargo fmt --check` (not `--all`) | `sdk-0a875da/sdk-fmt.log` | `FMT_EXIT:0` |
| package | `rm -rf target/horizon-mod && rm -f Cargo.lock && cargo horizon-mod build`, twice | `sdk-build-final1.log`, `sdk-build-final2.log` | **40,089 B**, SHA-256 `4e09f4f800aafaa4f6cc511a2b2781bf1c297484941668d67d9dbe3d831fed20`, **49 entries**, `CMP:identical`, no packed `Cargo.lock` |
| check | `cargo horizon-mod check rftoolsbase.hmod` | `sdk-check.log` | `CHECK_EXIT:0`; `rftoolsbase 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)` |
| fits, pin | `cargo horizon-mod fits … --build 0a875da` | `sdk-fits.log` | `0a875da  made for`, `FITS_EXIT:0` |
| fits, other | `cargo horizon-mod fits … --build 0b14f8a` | `sdk-fits-other.log` | `0b14f8a  fits`, `FITS_OTHER_EXIT:0` |
| gate | `python3 ports/tools/verify-sdk-package.py rftoolsbase` | — | **1/1**, 8/8 checks |
| logs | `check-e2e-logs.py rftoolsbase-e2e rftoolsbase --strict` | — | `logs=11 ok=11 missing-exit=0 bad-exit=0` |

`src/runtime.rs` packs as `c5293218484d…` (the tree file), and the package is unsigned local
build; `check`/`fits` both exit 0 on the delivered bytes.

### In-game: the three packages in one game (the differential)

`ports/_shared/evidence/rftoolsbase-e2e/three-game.sh` assembles a scratch folder with
`rftoolsbase` 0.2.2 + `rftoolsstorage` 0.2.5 + `rftoolspower` 0.1.1 (all checked at
`0a875da`/`e59da90`), runs the launcher's `native-build` and exports the version carrying all
three (log `three-game-build.log`):

```
rftoolsbase 0.2.2: 0a875da / e59da90
rftoolsstorage 0.2.5: 0a875da / e59da90
rftoolspower 0.1.1: 0a875da / e59da90
Checking 3 mods
Compiling rftoolsbase (1 of 3)
Compiling rftoolspower (2 of 3)
Compiling rftoolsstorage (3 of 3)
native-0a875da-ce46d69ed6e9 built and selected for its instance.
```

`native-mods.json` carries `0a875da91dbb0fdcb7ba8c42c11341aa229af51a rftoolsbase 0.2.2 1.99.1`,
and `horizon-native-mods/rftoolsbase/{horizon-mod.toml,Cargo.toml,README.md,src/lib.rs,src/runtime.rs}`
are `cmp`-identical to this port. `tools/e2e-craft.sh` (log
`ports/_shared/evidence/rftoolsbase-e2e/e2e-craft.log`) proves the thing this job exists for:

```
== A: the three packages are in one game ==
  in the inventory: rftoolsbase:machine_frame x1
  in the inventory: rftoolsstorage:storage_scanner x1
  in the inventory: rftoolspower:coalgenerator x1
== B: craft rftoolsstorage:storage_scanner (ToT / gFg / ToT, F = rftoolsbase:machine_frame) ==
  rftoolsstorage:storage_scanner pattern -> 'rftoolsstorage:storage_scanner x1'
  scanned: scanner=1, frame=0, grid empty
== C: a stone where rftoolsbase:machine_frame belongs crafts nothing ==
  same pattern with stone in the frame cell -> 'empty'
== D: craft rftoolspower:coalgenerator (cTc / cFc / cTc, c = #minecraft:coals) ==
  rftoolspower:coalgenerator pattern -> 'rftoolspower:coalgenerator x1'
  built: generator=1, frame=0, grid empty
PASS: …
EXIT:0
```

The take-result path shift-clicks slot 200, then asserts the inventory counts: the scanner and the
generator each appear **once**, and every ingredient — the frame, the torches, the ender pearls,
the gold, the coal — reads **0**. Phase C is the wrong-cell control: the same nine cells with
`minecraft:stone` where the frame belongs must leave the result empty while the stone stays in the
grid. **The leg cannot pass before the repin**: it needs the frame id, the recipe and a crafting
table in the same game, which is exactly what two mismatched pins made impossible.

Fail-capability, both recorded:

- `SELFTEST=1` (`e2e-craft-selftest.log`): `FAIL: SELFTEST: the frame cell crafted
  rftoolsstorage:storage_scanner x1, so the assertion is alive` + `EXIT:1`.
- `CONTROL=1` on the unmodded Nightly (`e2e-craft-control.log`): every `give` answers
  `{"error":"unknown item …"}` and A fails with `EXIT:1`.

The slice's own machine scenario was re-run on the same three-package game
(`e2e-repin-machines.log`): all of A–G pass (`PASS`, `EXIT:0`; 600/301/482 FE curve, the level-256
refusal, the dirt control), and its `SELFTEST=1` leg exits 1
(`e2e-repin-machines-selftest.log`).

### Coverage at this pin (slice 1)

| Family | Coverage |
|---|---|
| `rftoolsbase:machine_infuser` block + block entity + menu/tooltip | **tested** — A–G (`e2e-repin-machines.log`): placement, 60,000 FE buffer, 5-tick infusion, cost curve 600/301/482, level-256 refusal, non-shard control, placed-item level seeding |
| `rftoolsbase:machine_frame`/`machine_base`/`dimensionalshard` items | **tested** — present in the three-package game and consumed by both child recipes |
| the four carried recipes | **partly tested** — `machine_frame` is crafted as an ingredient by the child legs; the infuser recipe is not crafted in this leg (its result is the block, placed in A) |
| `rftoolsstorage:storage_scanner` recipe (`ToT`/`gFg`/`ToT`) | **tested** — B: result id/count, ingredients consumed, wrong-cell control empty |
| `rftoolspower:coalgenerator` recipe (`cTc`/`cFc`/`cTc`, `#minecraft:coals`) | **tested** — D: same assertions |
| shard ores/worldgen, information_screen, security_manager, infused_diamond/enderpearl, tools, Patchouli guide | **not ported** (gap list above, worldgen-last) |

### Gaps after the repin

1. The menu/screen and the two command item writes are still only covered by the retired `.hnmod`
   run and the unit tests; this leg drives gameplay, not the GUI (`--offscreen` is used, but no
   screenshot is taken).
2. The `machine_frame` recipe itself is not crafted in this leg; the two child recipes are the
   differential this job needed.
3. Everything the slice did not implement stays a gap (see “Deviations and gaps (slice 1)”).

### Next slice

**The remaining base items, starting with `information_screen` and the `infused_diamond` /
`infused_enderpearl` chain** — gap 12 of the slice-1 list. It fits the same `machines` tier: a
carried block/item plus one component and a small hook, no new engine surface. The in-game
assertion that would prove it: `ctl give rftoolsbase:infused_diamond 1`, craft it from
`rftoolsbase:dimensionalshard` + `minecraft:diamond` in a real table (assert result id/count and
both ingredients consumed), and read `ctl block` / the item's `rftoolsbase:infused` component after
placing the `information_screen` — with the same wrong-cell control.
