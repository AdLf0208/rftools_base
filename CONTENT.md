# RFToolsBase slice 1 — content (`.hmod`)

The carried content now lives at the package root (`assets/rftoolsbase/**`,
`data/**`) inside the flat SDK `.hmod`, beside `src/`: there is no separate
`content/` package any more (migrated 2026-11-18, see
`notes.md#migration-to-the-sdk-format`). The paths below name the retired
`content/` tree; each file is now the same file at the package root. The
assembler moved to
`ports/_shared/evidence/rftoolsbase/retired-hnmod-0.1.0/tools/assemble_content.py`.

`content/` was generated from the pack jar by
[`tools/assemble_content.py`](../_shared/evidence/rftoolsbase/retired-hnmod-0.1.0/tools/assemble_content.py); rerun it to rebuild.
The jar is `ports/_shared/licence-cache/rftoolsbase-1.21-6.0.11.jar`, SHA-1
`9614bdc1…5aee` (the pack record's own hash). Nothing is invented: models,
textures, blockstate, lang lines, recipes, advancements, loot table and the
`rftoolsbase:shards` tag come out of the jar, re-spelled only where 26.3
differs. The block/item/menu/tab declarations, the item definitions and the
`c:` tags the recipes name are written by the assembler, because a code-
registered 1.21.1 block has no datapack declaration to carry and the prepared
26.3 root ships no `c:` tags.

## Carried from the jar, byte for byte

| File | Why |
|---|---|
| `assets/rftoolsbase/models/block/machine_infuser.json` | the machine's cube model (front `block/infuser`) |
| `assets/rftoolsbase/models/block/machine_frame.json` | the frame's `cube_all` model (the item uses it) |
| `assets/rftoolsbase/models/block/machine_base.json` + `rftools_logicblock.json` | the base's five-pixel-tall model and its parent |
| `assets/rftoolsbase/models/item/{machine_infuser,machine_frame,machine_base,dimensionalshard}.json` | the four item models |
| `assets/rftoolsbase/textures/block/{infuser,+base/machinebottom,+base/machineside,+base/machinetop}.png` | the machine and frame textures |
| `assets/rftoolsbase/textures/item/dimensionalsharditem.png` | the shard |
| `assets/rftoolsbase/textures/gui/infuser.png` | upstream's GUI background — **carried but not drawn** (the menu uses the pin's plain panel) |
| `data/rftoolsbase/recipe/{machine_infuser,machine_frame,machine_base,dimensionalshard}.json` | the four recipes |
| `data/rftoolsbase/advancement/recipes/misc/*.json` | the four recipe unlocks |
| `data/rftoolsbase/loot_table/blocks/machine_infuser.json` | the machine's drop |
| `data/rftoolsbase/tags/item/shards.json` | `#rftoolsbase:shards` = the shard, which the tick tests |

## Patched, because 1.21.1 → 26.3 differs

| File | Patch |
|---|---|
| `blockstates/machine_infuser.json` | the jar lists six `facing` variants; the port's template (`minecraft:stonecutter`, the pin's only menued template) has four, so `facing=up/down` are dropped |
| the four recipes | `{"item": X}` → `"X"`, `{"tag": T}` → `"#T"` (26.3's ingredient form; the 1.21.1 object form is an unknown ingredient) |
| `loot_table/blocks/machine_infuser.json` | the entry's `copy_components` functions are removed (they name mcjtylib's item components, which this port does not have) and the pool gains the **singular typed** `{"type": "minecraft:survives_explosion"}` condition |

## Written by the assembler

| File | What |
|---|---|
| `assets/rftoolsbase/items/*.json` | 26.3 item definitions (`minecraft:model` → the carried item model) for the machine and the three items |
| `data/rftoolsbase/horizon/block/machine_infuser.json` | the block: `minecraft:stonecutter` template, hardness 2.0 / blast 2.0, correct tool, metal sound, full cube |
| `data/rftoolsbase/horizon/item/{machine_frame,machine_base,dimensionalshard}.json` | three items (`minecraft:stick` template, stack 64) |
| `data/rftoolsbase/horizon/menu/machine_infuser_menu.json` | 2 slots, 4 data slots |
| `data/rftoolsbase/horizon/creative_tab/rftoolsbase.json` | `itemGroup.rftoolsbase`, the four entries |
| `data/c/tags/item/{ingots/iron,dyes/blue,stones,glass_blocks}.json` | the convention tags the recipes name (the prepared 26.3 root ships none) |
| `data/minecraft/tags/block/mineable/pickaxe.json` | the machine |
| `assets/rftoolsbase/lang/en_us.json` | the four names, the tab, and upstream's two machine tooltip lines |

## Why the port draws its own screen

The pin's mod menu surface is a plain panel plus slot frames and numeric data
slots, not a texture blit (the same choice `charginggadgets` and `powah` made).
The 180 × 152 panel, the two slot positions and the inventory origin are
upstream's; the energy bar becomes the `energy/capacity FE` line and the two
new data slots (`infusing`, block `infused`) become text.

## Coverage

- One machine: `rftoolsbase:machine_infuser`.
- Three items: `rftoolsbase:machine_frame`, `rftoolsbase:machine_base`,
  `rftoolsbase:dimensionalshard`.
- Four recipes, four advancements, one loot table, one pickaxe tag, four
  convention tags, one creative tab.
