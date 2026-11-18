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
