# Working on a native Horizon mod

> **Build slots (v2, 2026-11-09):** every build/test/e2e goes through the
> campaign runner (`ports/tools/build-slot.sh` from the campaign root,
> `../tools/build-slot.sh` from this port). It **blocks in the kernel** until a
> slot is free — a freed slot is taken in ~0.07 s — so **never poll**: no
> `while ! slot; sleep` loop, no background build. It sets `CARGO_BUILD_JOBS` per
> step (build/test 3, stage 2, e2e 1), journals grants/releases, and a timeout
> exits **75**. Read `../_shared/build-slots-usage.md` before running anything;
> observe with `status` / `watch` / `events` / `stats`, never with `ps`+`sleep`.
> Never pass `--target-dir`, never run a bare `cargo` here.

Read README.md and compatibility.json before editing. Keep this repository
separate from Horizon and horizon-mixins. Only export committed pinned Horizon
sources into ignored target trees; never modify the original checkout or player
resources. Do not commit binaries, exported sources or Minecraft assets.

The Cargo library and injected runtime module share runtime.rs. Keep helper
functions small and gameplay logic testable there. All AST changes go through
horizon-mixins plan/apply; do not add string-based Rust source rewriting.

Validate with cargo fmt, clippy with warnings denied, cargo test and
python3 tools/test_template.py. For bundle/compatibility changes also run
python3 tools/test_stage.py and python3 tools/horizon.py check/test against the
pinned source export. Keep validation claims and supported revisions honest.
