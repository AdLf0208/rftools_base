#!/usr/bin/env bash
# In-game differential for the repin: the three RFTools packages in ONE game,
# crafting the two recipes that name `rftoolsbase:machine_frame`.
#
# Before the repin no such game could exist: `rftoolsstorage` and `rftoolspower`
# pin `0a875da` and `rftoolsbase` still pinned `0b14f8a`, and the launcher refuses
# a game whose packages disagree on `[native] horizon`. `rftoolsstorage` 0.2.5
# recorded `blocked(rftoolsbase:machine_frame)` for this reason, and
# `rftoolspower`'s own e2e phase M proved the same recipe crafted nothing.
#
# This leg is the differential for that job and cannot pass before the repin:
#
#  A  the three packages are in one game (`give` answers for each id)
#  B  `rftoolsstorage:storage_scanner` crafts in a real crafting table from
#     `ToT` / `gFg` / `ToT` (F = `rftoolsbase:machine_frame`): the result id and
#     count are read from the table, the stack is taken, and every ingredient is
#     consumed
#  C  control: the same pattern with `minecraft:stone` in the frame cell crafts
#     nothing and leaves the ingredients in the grid
#  D  `rftoolspower:coalgenerator` crafts from `cTc` / `cFc` / `cTc`
#     (`#minecraft:coals`), result id and count asserted, ingredients consumed
#
# SELFTEST=1 inverts B (it requires the frame cell to craft nothing) and exits 1.
# CONTROL=1 runs A on the same Nightly WITHOUT the packages: the ids are unknown,
# so A fails and the log says why.
#
# GAME/CTL/CONTROL_GAME/RES/E2E_TMP come from the three-package export
# (`ports/_shared/evidence/rftoolsbase-e2e/three-game.sh env`).
set -Eeuo pipefail
export LC_ALL=C

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/rftoolsbase
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
E2E_TMP=${E2E_TMP:-/tmp/opencode}
SOCK=${SOCK:-$E2E_TMP/rftoolsbase-craft.sock}
SELFTEST=${SELFTEST:-0}
CONTROL=${CONTROL:-0}
KEEP=${KEEP:-0}
WORLD=RFToolsBaseCraftE2E
X0=430
SX=$((X0 + 2))
FRAME=rftoolsbase:machine_frame
SCANNER=rftoolsstorage:storage_scanner
GENERATOR=rftoolspower:coalgenerator

[ -n "$CTL" ] || { echo "set CTL= to this pin's horizon-ctl (three-game.sh env)" >&2; exit 1; }
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
if [ "$CONTROL" = 1 ]; then
    [ -n "${CONTROL_GAME:-}" ] || { echo "set CONTROL_GAME= to the unmodded Nightly game" >&2; exit 1; }
    GAME=$CONTROL_GAME
fi
[ -n "$GAME" ] || { echo "set GAME= to the three-package game (three-game.sh env)" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "$E2E_TMP/rftoolsbase-craft-home.XXXXXX")
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 40 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 30 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    echo "EXIT:$status"
}
trap cleanup EXIT

count_item() { ctl inventory | jq -r --arg i "$1" '.counts[$i] // 0'; }

echo "== launch (three packages in one game, The End arena) =="
"$CTL" --socket "$SOCK" launch --game "$GAME" --world "$WORLD" --seed 20261105 \
    --creative --offscreen --wait --resources "$RES" --game-dir "$SCRATCH/game" >/dev/null
ctl wait-ready 180 >/dev/null
ctl wait 40 >/dev/null

echo "== A: the three packages are in one game =="
for id in "$FRAME" "$SCANNER" "$GENERATOR"; do
    reply=$(ctl give "$id" 1 2>&1) || true
    echo "  give $id -> $reply"
done
# `ctl give` lands on the next tick: read the inventory after it has.
ctl wait 6 >/dev/null
for id in "$FRAME" "$SCANNER" "$GENERATOR"; do
    have=$(count_item "$id")
    echo "  in the inventory: $id x$have"
    [ "$have" -ge 1 ] || fail "$id is not in this game (the three packages are not all loaded)"
done
ctl command "clear @s" >/dev/null
ctl wait 4 >/dev/null

# --- the End arena, one floor and one crafting table at a time ---------------
arena() {
    ctl command "execute in minecraft:the_end run tp @s $X0.5 104 0.5" >/dev/null
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$X0" 100 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill $((X0 - 4)) 100 -6 $((X0 + 8)) 100 6 minecraft:stone" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 4)) 101 -6 $((X0 + 8)) 108 6 minecraft:air" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 101 0.5" >/dev/null
    ctl wait 20 >/dev/null
    [ "$(ctl block "$X0" 100 0 | jq -r '.name // .block // empty')" = "minecraft:stone" ] \
        || fail "the arena floor is not stone: $(ctl block "$X0" 100 0 | jq -c '.')"
}

arena

# --- the crafting table: grid 100..108 row-major, result 200 ----------------
open_table() { # x y z
    ctl close >/dev/null 2>&1 || true
    ctl wait 2 >/dev/null
    ctl setblock "$1" "$2" "$3" minecraft:air >/dev/null
    ctl setblock "$1" "$2" "$3" minecraft:crafting_table >/dev/null
    ctl wait 2 >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $(( $1 - 1 )).5 $2 $(( $3 )).5" >/dev/null
    ctl wait 4 >/dev/null
    ctl interact "$1" "$2" "$3" >/dev/null || fail "the crafting table did not take the click"
    ctl wait 6 >/dev/null
    local grid
    grid=$(ctl screen | jq '[.slots[]? | select(.slot >= 100)] | length')
    [ "$grid" = 10 ] || fail "the crafting table did not open its grid ($grid grid slots)"
}

place_one() { # place_one ITEM CELL — pick the stack up, right-click one into the cell, put it back
    local item=$1 cell=$2 src
    src=$(ctl screen | jq -r --arg i "$item" '[.slots[]? | select(.slot < 36 and .item == $i) | .slot] | first // ""')
    [ -n "$src" ] || fail "no $item in the inventory for the recipe"
    ctl click-slot "$src" >/dev/null
    ctl wait 1 >/dev/null
    ctl click-slot "$cell" right >/dev/null
    ctl wait 1 >/dev/null
    ctl click-slot "$src" >/dev/null
    ctl wait 1 >/dev/null
}

craft_result() { ctl screen | jq -r '[.slots[]? | select(.slot == 200 and ((.item // "") != "")) | "\(.item) x\(.count)"] | first // "empty"'; }
grid_items() { ctl screen | jq -c '[.slots[]? | select(.slot >= 100 and .slot <= 108) | select((.item // "") != "") | {slot, item}]'; }

give_set() { # give_set ITEM COUNT ...
    ctl command "clear @s" >/dev/null
    while [ $# -ge 2 ]; do
        ctl give "$1" "$2" >/dev/null || fail "cannot give $1"
        shift 2
    done
    ctl wait 6 >/dev/null
}

take_result() {
    ctl click-slot 200 shift >/dev/null || fail "taking the craft result failed"
    ctl wait 4 >/dev/null
}

# ---------------------------------------------------------------------------
# B. the scanner recipe that names the frame
# ---------------------------------------------------------------------------
echo "== B: craft $SCANNER (ToT / gFg / ToT, F = $FRAME) =="
ctl command "gamemode survival @s" >/dev/null
ctl wait 3 >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 3 >/dev/null
give_set minecraft:redstone_torch 4 minecraft:ender_pearl 2 minecraft:gold_ingot 2 "$FRAME" 1
open_table "$SX" 101 0
place_one minecraft:redstone_torch 100
place_one minecraft:ender_pearl 101
place_one minecraft:redstone_torch 102
place_one minecraft:gold_ingot 103
place_one "$FRAME" 104
place_one minecraft:gold_ingot 105
place_one minecraft:redstone_torch 106
place_one minecraft:ender_pearl 107
place_one minecraft:redstone_torch 108
scanner_result=$(craft_result)
echo "  $SCANNER pattern -> '$scanner_result'"
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the machine-frame cell crafts nothing)"
    [ "$scanner_result" = "empty" ] || fail "SELFTEST: the frame cell crafted $scanner_result, so the assertion is alive"
else
    [ "$scanner_result" = "$SCANNER x1" ] \
        || fail "the frame cell did not craft the scanner (got '$scanner_result'; grid $(grid_items))"
fi
take_result
[ "$(count_item "$SCANNER")" -eq 1 ] || fail "the crafted scanner is not in the inventory ($(count_item "$SCANNER"))"
for spent in minecraft:redstone_torch minecraft:ender_pearl minecraft:gold_ingot "$FRAME"; do
    [ "$(count_item "$spent")" -eq 0 ] || fail "$spent was not consumed ($(count_item "$spent") left)"
done
[ "$(grid_items)" = "[]" ] || fail "the grid was not emptied: $(grid_items)"
echo "  scanned: scanner=$(count_item "$SCANNER"), frame=$(count_item "$FRAME"), grid empty"
ctl close >/dev/null 2>&1 || true
ctl wait 3 >/dev/null

# ---------------------------------------------------------------------------
# C. control: a stone where the frame belongs
# ---------------------------------------------------------------------------
echo "== C: a stone where $FRAME belongs crafts nothing =="
ctl command "clear @s" >/dev/null
ctl wait 3 >/dev/null
give_set minecraft:redstone_torch 4 minecraft:ender_pearl 2 minecraft:gold_ingot 2 minecraft:stone 1
open_table "$SX" 101 -4
place_one minecraft:redstone_torch 100
place_one minecraft:ender_pearl 101
place_one minecraft:redstone_torch 102
place_one minecraft:gold_ingot 103
place_one minecraft:stone 104
place_one minecraft:gold_ingot 105
place_one minecraft:redstone_torch 106
place_one minecraft:ender_pearl 107
place_one minecraft:redstone_torch 108
control_result=$(craft_result)
echo "  same pattern with stone in the frame cell -> '$control_result'"
[ "$control_result" = "empty" ] || fail "a stone in the frame cell crafted '$control_result'"
[ "$(ctl screen | jq -r '[.slots[]? | select(.slot == 104) | .item][0] // "empty"')" = "minecraft:stone" ] \
    || fail "the control's stone left the grid, so the control did not hold"
ctl close >/dev/null 2>&1 || true
ctl wait 3 >/dev/null

# ---------------------------------------------------------------------------
# D. the generator recipe that names the frame
# ---------------------------------------------------------------------------
echo "== D: craft $GENERATOR (cTc / cFc / cTc, c = #minecraft:coals) =="
ctl command "clear @s" >/dev/null
ctl wait 3 >/dev/null
give_set minecraft:coal 6 minecraft:redstone_torch 2 "$FRAME" 1
open_table "$SX" 101 4
place_one minecraft:coal 100
place_one minecraft:redstone_torch 101
place_one minecraft:coal 102
place_one minecraft:coal 103
place_one "$FRAME" 104
place_one minecraft:coal 105
place_one minecraft:coal 106
place_one minecraft:redstone_torch 107
place_one minecraft:coal 108
generator_result=$(craft_result)
echo "  $GENERATOR pattern -> '$generator_result'"
[ "$generator_result" = "$GENERATOR x1" ] \
    || fail "the frame cell did not craft the generator (got '$generator_result'; grid $(grid_items))"
take_result
[ "$(count_item "$GENERATOR")" -eq 1 ] || fail "the crafted generator is not in the inventory ($(count_item "$GENERATOR"))"
for spent in minecraft:coal minecraft:redstone_torch "$FRAME"; do
    [ "$(count_item "$spent")" -eq 0 ] || fail "$spent was not consumed ($(count_item "$spent") left)"
done
[ "$(grid_items)" = "[]" ] || fail "the generator grid was not emptied: $(grid_items)"
ctl command "gamemode creative @s" >/dev/null
ctl wait 3 >/dev/null
echo "  built: generator=$(count_item "$GENERATOR"), frame=$(count_item "$FRAME"), grid empty"

echo "PASS: the three packages share one game; $SCANNER crafts in a real table from ToT/gFg/ToT with $FRAME in the centre (result $SCANNER x1, every ingredient consumed, grid empty); a stone in the frame cell crafts nothing and stays in the grid; $GENERATOR crafts from cTc/cFc/cTc with the same frame (result $GENERATOR x1, coal, torches and frame consumed)"
