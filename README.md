# RFToolsBase — Horizon port (slice 1: the machine family and the Machine Infuser)

A native Horizon port of **RFToolsBase 1.21-6.0.11**
(`McJtyMods/RFToolsBase`, **MIT**): the machine frame/base family, the
dimensional shard, and the Machine Infuser — a 60,000 FE machine that spends
one shard and `600 * (2 - infused/256) / 2` FE to raise a machine's infused
level by one, up to 256, which makes its own later infusions cheaper.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
MIT is licence rank 0 (permissive); approval is still required.

Provenance, hashes and the upstream commit are in
`ports/_shared/carry-work/rftoolsbase/SOURCE.md`; the jar inventory and the
behaviour reading are in `FINDINGS.md`; the ladder, deviations, gaps and the
migration are in `notes.md`; the content decisions are in `CONTENT.md`.

## Layout (flat SDK package, mod API 0.2)

```
horizon-mod.toml                   the only manifest and identity: id rftoolsbase, 0.2.2,
                                   api 0.2, [native] 0a875da / e59da90
Cargo.toml                         one crate; horizon-mod-api from .horizon/sdk/
src/lib.rs                         the one impl Hooks (register, component_types, commands),
                                   the machine and item Parts, and 7 glue tests
src/runtime.rs                     pure: the shard cost curve, the infusion cycle, data slots
                                   (byte-identical to the retired mixins file, 14 tests)
assets/rftoolsbase/**              carried: blockstate, models, textures, item definitions, lang
data/rftoolsbase/**                carried: block, items, menu, tab, recipes, loot,
                                   advancements, the rftoolsbase:shards tag
rftoolsbase.hmod                   the delivered package: code and content in one file
tools/e2e.sh                       the in-game scenario (needs a launcher-built game)
tools/e2e-craft.sh                 the three-package differential: the two recipes that
                                   name rftoolsbase:machine_frame craft in one game
```

The retired `.hnmod`, its `mixins/` tree, the old `content/` package and their
build tools are kept as evidence in
`ports/_shared/evidence/rftoolsbase/retired-hnmod-0.1.0/`.

## What slice 1 does

- `rftoolsbase:machine_infuser`: an insert-only 60,000 FE buffer (600 a call),
  two slots (shards, one infusable machine), a five-tick infusion that spends
  one shard and 600 FE at level 0 (301 at 255, 300 at 256) and raises the item's
  `rftoolsbase:infused` component by one; the placed block carries that level
  and infuses more cheaply. A 180×152 screen with the two slots, an FE line and
  the infusion readouts.
- `rftoolsbase:machine_frame`, `rftoolsbase:machine_base`,
  `rftoolsbase:dimensionalshard` and the four recipes that build the chain.

## Build

```sh
cd ports/rftoolsbase
cargo horizon-mod build                 # fetches .horizon/sdk/ and packs target/horizon-mod/
CARGO_INCREMENTAL=0 cargo test          # 22 tests (14 runtime + 8 glue)
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
cargo fmt --check                       # not --all: that reads the fetched SDK
rm -f Cargo.lock && cargo horizon-mod build && cp target/horizon-mod/rftoolsbase.hmod .
cargo horizon-mod check rftoolsbase.hmod
cargo horizon-mod fits rftoolsbase.hmod --build 0a875da
```

Every compile/test/check step goes through the campaign registry with
`BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-rfb --port
ports/rftoolsbase --step <step> -- bash -c 'cd ports/rftoolsbase && …'`; the
`cd` is required because the runner does not enter the port. The game legs need
a game built with the mod (`horizon-launcher native-build . --instance <i>
--trust-native-code`) pointed at with `GAME=`:

```sh
ports/tools/build-slot.sh e2e --job rfb-e2e --port ports/rftoolsbase -- tools/e2e.sh
ports/tools/build-slot.sh e2e --job rfb-e2e-selftest --port ports/rftoolsbase -- env SELFTEST=1 tools/e2e.sh
```
