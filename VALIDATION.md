# Validation — RFToolsBase slice 1 (flat SDK package, mod API 0.2)

Pin: Horizon **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (nightly 2026-11-18),
horizon-mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, mod API `0.2`, game
26.3. Every compile/test/check step ran inside `ports/tools/build-slot.sh` with
`CARGO_INCREMENTAL=0` and `BUILD_LOCAL=1` (`cargo horizon-mod` needs the port's own
`.horizon/sdk/`, which the second machine does not have). Logs:
`ports/_shared/evidence/rftoolsbase/sdk-0b14f8a/`.

| # | Step | Command (inside the lease) | Evidence | Result |
|---|---|---|---|---|
| 1 | runtime carried over | `cp` + `cmp` against the retired `mixins/rftoolsbase/runtime.rs` | — | **byte-identical**, SHA-256 `e6ffec7f…4425` |
| 2 | runtime tests | `cargo test runtime::` | `sdk-runtime-tests.log` | **14 passed**, 0 failed |
| 3 | all tests | `cargo test` | `sdk-test.log` | **21 passed** (14 runtime + 7 glue), 0 failed |
| 4 | lints | `cargo clippy --all-targets -- -D warnings` | (rerun in `notes.md`) | exit 0, 0 warnings |
| 5 | formatting | `cargo fmt --check` (not `--all`) | — | clean; `cmp` still byte-identical afterwards |
| 6 | package | `rm -f Cargo.lock && cargo horizon-mod build` | `sdk-build.log` | `target/horizon-mod/rftoolsbase.hmod` |
| 7 | registry check | `cargo horizon-mod check rftoolsbase.hmod` | `sdk-check.log` | **exit 0** |
| 8 | surface used | `cargo horizon-mod check rftoolsbase.hmod --uses` | `sdk-uses.json` | 354 entries |
| 9 | fits the pinned build | `cargo horizon-mod fits rftoolsbase.hmod --build nightly` | `sdk-fits.log` | `0b14f8a made for`, **exit 0** |
| 10 | hash | `sha256sum rftoolsbase.hmod` | `sdk-sha256.txt` | see `notes.md` (rebuilt twice, same bytes) |
| 11 | `runtime.rs` in the package | `unzip -p rftoolsbase.hmod src/runtime.rs \| sha256sum` | — | `e6ffec7f…4425`, the retired file |
| 12 | e2e | needs a launcher-built game | — | **not re-run here**; `tools/e2e.sh` updated to the flat layout |

The 0.1.0 `.hnmod` ladder (red 0/14 → green 14/14, stage/check/test/build, 4 e2e
logs with `EXIT:` lines, the embedded-content legs) is history and is recorded in
the retired tree under
`ports/_shared/evidence/rftoolsbase/retired-hnmod-0.1.0/` and in `notes.md`.

## Rebuild determinism

The flat package was built twice from the same sources (the second build after the
documentation rewrite, with `Cargo.lock` removed and `.horizon/sdk/` in place) and
reproduced its bytes: **39,218 B**, SHA-256
`f754cd0db35461e6617f65db031efdf9c6a85151c4238e8c6394cbc18204ac20`. Logs:
`sdk-build.log`, `sdk-build-repeat.log`.

## Repin to `0a875da` — 0.2.2 (2026-11-21)

Horizon **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** / mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, game 26.3. `.horizon/sdk` was
deleted before the first build and the tool printed the new commit. No source
change; version 0.2.2 in both manifests; `README.md` updated (it is packed).
Evidence: `ports/_shared/evidence/rftoolsbase/sdk-0a875da/` and
`ports/_shared/evidence/rftoolsbase-e2e/`.

| # | Step | Command | Evidence | Result |
|---|---|---|---|---|
| 1 | SDK refresh | `rm -rf .horizon/sdk && cargo horizon-mod build` | `sdk-build-1.log` | `SDK 0a875da (build 0a875da91…)` |
| 2 | tests | `cargo test` | `sdk-test.log` | **22 passed**, 0 failed |
| 3 | lints | `cargo clippy --all-targets -- -D warnings` | `sdk-clippy.log` | exit 0, 0 warnings |
| 4 | formatting | `cargo fmt --check` | `sdk-fmt.log` | `FMT_EXIT:0` |
| 5 | package ×2 | `rm -rf target/horizon-mod && rm -f Cargo.lock && cargo horizon-mod build` | `sdk-build-final1.log`, `sdk-build-final2.log` | **40,089 B**, SHA-256 `4e09f4f8…1fed20`, 49 entries, `CMP:identical`, no lock |
| 6 | registry check | `cargo horizon-mod check rftoolsbase.hmod` | `sdk-check.log` | **exit 0** |
| 7 | fits, pin | `cargo horizon-mod fits … --build 0a875da` | `sdk-fits.log` | `0a875da  made for`, exit 0 |
| 8 | fits, other | `cargo horizon-mod fits … --build 0b14f8a` | `sdk-fits-other.log` | `0b14f8a  fits`, exit 0 |
| 9 | gate | `python3 ports/tools/verify-sdk-package.py rftoolsbase` | — | **1/1** |
| 10 | three-package game | `ports/_shared/evidence/rftoolsbase-e2e/three-game.sh` | `three-game-build.log` | `native-0a875da-ce46d69ed6e9`, `rftoolsbase 0.2.2` identity |
| 11 | craft differential | `tools/e2e-craft.sh` | `e2e-craft.log` | `PASS` + `EXIT:0` (scanner and generator crafted, wrong-cell control empty) |
| 12 | selftest | `SELFTEST=1 tools/e2e-craft.sh` | `e2e-craft-selftest.log` | `FAIL` + `EXIT:1` |
| 13 | control | `CONTROL=1 tools/e2e-craft.sh` | `e2e-craft-control.log` | unknown ids, `FAIL` + `EXIT:1` |
| 14 | machine regression | `tools/e2e.sh` on the same game | `e2e-repin-machines.log` | `PASS` + `EXIT:0` |
| 15 | machine selftest | `SELFTEST=1 tools/e2e.sh` | `e2e-repin-machines-selftest.log` | `FAIL` + `EXIT:1` |
| 16 | log gate | `check-e2e-logs.py rftoolsbase-e2e rftoolsbase --strict` | — | `logs=11 ok=11` |
