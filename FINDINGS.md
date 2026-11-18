# RFToolsBase — findings (jar inventory + upstream behaviour)

Ported from **RFToolsBase 1.21-6.0.11** (`rftoolsbase-1.21-6.0.11.jar`,
SHA-1 `9614bdc12d6c633801b254a983d30dd4cbeb5aee`, SHA-256
`5195ba530e6cf9ba61c9954a3297679e6d29aa1b6182e27ae14ea43463dd4b00`), the exact
ATM10 file (`ports/_shared/licence-cache/atm10-modlist.json` entry:
`modrinthHash` = the same SHA-1). Licence read from the jar's
`META-INF/neoforge.mods.toml`: `MIT License
https://github.com/McJtyMods/RFToolsBase/blob/1.16/LICENCE` → **MIT, licence rank
0**. Local-only until the maintainer approves publishing (ARCHITECTURE.md §8).

Upstream was read from tag/branch `1.21_neo`, commit
`4f7e15e64c7eb05365895773aa8f20f78ec4ac15` (2026-06-16, "Bump McJtyLib
dependency", `gradle.properties` version `1.21-6.0.11`), plus McJtyLib
`1.21_neo` `ef393e608eebc188a6e1decbf110506a55cc0725`. Analysis only: this is a
clean-room rewrite, no Java was copied or translated and no jar was decompiled.

## Jar inventory (the artefact wins)

Registered blocks (`RBLOCKS.registerBlock`):

| id | what it is | in this slice |
|---|---|---|
| `rftoolsbase:machine_infuser` | the Machine Infuser: block entity, 60,000 FE, two slots, five-tick infusion | **yes** |
| `rftoolsbase:information_screen` | a panel that displays a powercell's information | no — slice 2 |
| `rftoolsbase:dimensionalshard_overworld` / `_nether` / `_end` | the three ores | no — worldgen-last ruling |

Registered items (`ITEMS.register`):

| id | what it is | in this slice |
|---|---|---|
| `rftoolsbase:machine_frame` | the frame every RFTools machine is built from (a plain item in 1.21, **not** a block, despite the jar's leftover blockstate/model) | **yes** |
| `rftoolsbase:machine_base` | the base of the information screen (plain item) | **yes** |
| `rftoolsbase:dimensionalshard` | the infusion fuel, in `rftoolsbase:shards` and `c:dusts` | **yes** |
| `rftoolsbase:infused_diamond`, `infused_enderpearl` | crafting materials for the other RFTools mods | no — not part of the machine family |
| `rftoolsbase:crafting_card`, `filter_module`, `tablet`, `tablet_filled`, `manual`, `smartwrench`, `smartwrench_select` | tools and guides | no — slice 2+ |

The jar's `lang/en_us.json` also carries legacy keys (`security_manager`,
`network_monitor`, `teleport_probe`, `shard_wand`, `orphaning_card`,
`security_card`, `developers_delight`) whose items are **not registered** in
1.21 — they are not content and are not carried.

Assets present for this slice: blockstates/models for the machine and the two
items (the leftover `machine_base`/`machine_frame` block models are used as the
items' models), six textures, `textures/gui/infuser.png` (256×256, upstream's
GUI background — the port draws a plain panel instead), `en_us.json`, four
recipes, four recipe advancements, one loot table, `data/rftoolsbase/tags/item/
shards.json`, the pickaxe tag, the four `neoforge/biome_modifier` + worldgen
files (ores, not carried), a Patchouli book (not carried: Patchouli is a library
this campaign has not mapped).

## The Machine Infuser (behaviour read from upstream)

`MachineInfuserTileEntity` extends `TickingTileEntity`; `MachineInfuserConfiguration`
declares, in the `infuser` server config section:

| key | default | meaning |
|---|---:|---|
| `maxInfuse` | 256 | shards a machine can take |
| `usePerTick` (`RFPERTICK`) | 600 | FE an infusion costs at level 0 |
| `infuserMaxRF` (`MAXENERGY`) | 60,000 | the buffer |
| `infuserRFPerTick` (`RECEIVEPERTICK`) | 600 | the most one insert moves |

`tickServer`:

1. if `infusing > 0`: decrement; at zero call `finishInfusing(slot 1)`;
2. else if slot 0 holds a `rftoolsbase:shards` item **and** slot 1 holds an
   infusable machine (`BlockItem` of a `BaseBlock` whose `isInfusable()` is true
   and whose `getInfused()` is below `maxInfuse`), call `startInfusing`;
3. `startInfusing` computes `rf = (int)(RFPERTICK * (2.0f - infusedFactor) /
   2.0f)` with `infusedFactor = blockInfused / 256`, returns if
   `energy < rf`, else spends `rf`, takes one shard and sets `infusing = 5`;
4. `finishInfusing` sets the slot-1 item's `mcjtylib:infusable` component to
   `infused + 1`.

Energy: `GenericEnergyStorage(this, true, 60,000, 600)` — a receiver that never
extracts. Slots: `ContainerFactory` slot 0 accepts shards, slot 1 accepts
infusable machines and is also an output; slot 1's limit is 1, slot 0's 64.
The menu (`GenericContainer`, `GuiMachineInfuser`) is 180×152 with slots at
(64, 24) and (118, 24) and the player inventory at (10, 70); it draws an energy
bar at (10, 7, 8, 54).

**The infuser's own level.** The block is built by `BaseBlock`, whose
`infusable` flag decides whether an infuser can raise it. In 1.20 the machine's
registration went through `MachineInfuserTileEntity.createBlock()`, which calls
`.infusable()`; in 1.21 `MachineInfuserModule` builds a plain
`new BaseBlock(new BlockBuilder().tileEntitySupplier(...))`, so the flag is
**false** and RFToolsBase has no infusable block at all — the infuser can only
raise machines from its sibling mods (RFToolsUtility/RFToolsPower call
`.infusable()`). This port restores the 1.20 intent, so the machine can infuse
its own block item; that is what makes the cost curve observable in this pack
(see `PLAN.md`, decision 1).

`BaseBlock.intAddInformation` shows `Infused: N%` (`infused * 100 / 256`) on an
item that carries the component, and `Energy: N rf` when it carries energy.

## Slice boundary and why

- **In:** the base machine family — `machine_frame`, `machine_base`,
  `dimensionalshard`, the recipes that build them and the machine — plus the
  Machine Infuser as the one working machine, with its block entity, buffer,
  container, menu, placement and tooltip.
- **Out, with reasons:** the three ores and their worldgen (worldgen-last
  ruling, ARCHITECTURE.md §8); `information_screen` (a second machine, its
  display reads a *powercell* capability this pack has no provider for; next
  slice); every tool item (client/GUI-heavy, none needed by the machine chain);
  the Patchouli manual (unmapped library); `infused_diamond`/`infused_enderpearl`
  (craft-only materials for other mods' recipes).
- The machine's target in this port is its own block item, as upstream's 1.20
  registration intended; other RFTools mods' machines are not in this port and
  are recorded as a gap.
