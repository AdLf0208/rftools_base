# RFTools Base — a Horizon port

**RFTools Base** is a Minecraft mod. Base mod for all RFTools and tech mods. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [McJtyMods/RFToolsBase](https://github.com/McJtyMods/RFToolsBase) — licence **MIT**.

| | |
|---|---|
| Upstream mod | RFTools Base ([source](https://github.com/McJtyMods/RFToolsBase)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `rftoolsbase.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What slice 1 does

- `rftoolsbase:machine_infuser`: an insert-only 60,000 FE buffer (600 a call),
  two slots (shards, one infusable machine), a five-tick infusion that spends
  one shard and 600 FE at level 0 (301 at 255, 300 at 256) and raises the item's
  `rftoolsbase:infused` component by one; the placed block carries that level
  and infuses more cheaply. A 180×152 screen with the two slots, an FE line and
  the infusion readouts.
- `rftoolsbase:machine_frame`, `rftoolsbase:machine_base`,
  `rftoolsbase:dimensionalshard` and the four recipes that build the chain.

## Not yet ported

- **The infuser's block is infusable.** Its 1.21 registration drops upstream's
- **`facing` is horizontal (4 values).** `minecraft:stonecutter`, the pin's
- **`rftoolsbase:infused` is a component, not `mcjtylib:infusable`.** The
- **A broken machine loses its FE and its slots.** Upstream's loot table copies
- **The block's level is stored in the block entity, not a component.** As
- **No GUI texture or bars.** The carried `textures/gui/infuser.png` is not
- **No per-slot insert rules.** Upstream refuses a non-shard in slot 0 and a
- **A stack of more than one in slot 1 is treated as one machine item**

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `rftoolsbase.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

RFTools Base was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/McJtyMods/RFToolsBase

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run rftoolsbase` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/rftoolsbase/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
