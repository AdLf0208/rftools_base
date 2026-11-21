//! RFToolsBase for Horizon — the Machine Infuser and the machine family.
//!
//! [`runtime`] is the port's pure logic (the shard cost curve, the five-tick infusion cycle and
//! the menu's data slots) and carries its own 14 tests; this file is only the game glue.
//!
//! The machine is a block entity with a 60,000 FE buffer, two slots (a dimensional shard in slot
//! 0, one infusable machine in slot 1) and a server tick: a shard costs
//! `600 * (2 - infused/256) / 2` FE, where `infused` is the **block's** own level, and five ticks
//! later the item in slot 1 gains one level, up to 256. The block's level is seeded from the item
//! it was placed from, so an infused machine infuses more cheaply. The block, its item and the
//! machine family are the carried content in `assets/` and `data/` of this same package.
pub mod runtime;

use std::array;

use mod_api::{
    self, BlockEntityPart, BlockEntityRef, BlockEntityTickEvent, BlockEntityTickReply,
    BlockEntityTicksEvent, BlockPart, ClientHost, Command, CommandArgumentValue, CommandSource,
    ComponentTypesEvent, Compound, DimensionKey, Flow, Hooks, Host, HostTransaction, Identifier,
    ItemPart, ItemStack, LevelRef, ModContainer, ModEnergyBuffer, OnBlockPlacedByEvent, PlayerRef,
    RegisterCommandsEvent, Registrar, RegistrationContext, RunModCommandEvent, Screen,
    ScreenDrawEvent, ScreenEvent, ServerHost, Tag, Text, TooltipEvent,
};

use runtime::{CAPACITY, INFUSING_TICKS, MAX_INFUSE, MAX_INSERT, Machine};

/// The machine block and its item (`MachineInfuserModule.MACHINE_INFUSER`).
const MACHINE_INFUSER: &str = "rftoolsbase:machine_infuser";
/// The block entity type this mod declares for the machine, in its own namespace. It is the
/// same string as the block id because the machine is the family's only block entity.
const MACHINE_TYPE: &str = "machine_infuser";
/// The item tag `MachineInfuserTileEntity.isShard` tests (`VariousModule.SHARDS`).
const SHARD_TAG: &str = "rftoolsbase:shards";
/// mcjtylib's `mcjtylib:infusable` component in the pin's own form.
const COMPONENT_INFUSED: &str = "rftoolsbase:infused";
/// The menu the content declares (`data/rftoolsbase/horizon/menu/machine_infuser_menu.json`).
const MENU: &str = "rftoolsbase:machine_infuser_menu";
/// Upstream's slot order (`MachineInfuserTileEntity.SLOT_SHARDINPUT/…OUTPUT`).
const SHARD_SLOT: usize = 0;
const TARGET_SLOT: usize = 1;
const SLOT_COUNT: usize = 2;
/// Upstream's saved keys; the FE amount is the engine's `energy` int.
const KEY_ENERGY: &str = "energy";
const KEY_INFUSING: &str = "infusing";
const KEY_INFUSED: &str = "infused";
/// The tooltip's colour, opaque yellow as `BaseBlock.intAddInformation` adds it.
const TOOLTIP_YELLOW: u32 = 0xFFFF_FF55;
/// The screen's text colour, the same grey the retired port drew.
const SCREEN_GREY: u32 = 0xFF40_4040;

// ---------------------------------------------------------------------------
// The declarations: the block entity type, its energy and slots, and the item
// ---------------------------------------------------------------------------

/// The machine's Part: its tick, its menu layout and its readouts.
struct MachinePart;

/// The machine item's Part: its tooltip and the level a placed item seeds.
struct MachineItem;

/// The item domain: two slots — a shard in 0, the machine in 1 — with every face reaching both
/// and automation allowed to take the finished machine out of slot 1
/// (`MachineInfuserTileEntity`'s `WorldlyContainer`).
fn container() -> ModContainer {
    ModContainer {
        slots: SLOT_COUNT,
        faces: array::from_fn(|_| vec![SHARD_SLOT, TARGET_SLOT]),
        inputs: vec![SHARD_SLOT, TARGET_SLOT],
        outputs: vec![TARGET_SLOT],
        menu: Some(MENU.to_owned()),
    }
}

/// The energy domain (`GenericEnergyStorage(this, true, 60000, 600)`): 60,000 FE, 600 a call,
/// inserts only on every face. The amount is the `energy` int in the saved compound.
fn buffer() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: CAPACITY,
        max_insert: MAX_INSERT,
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
    }
}

/// The machine's two slots in its 180 x 152 panel, at upstream's `CONTAINER_FACTORY` pixels
/// (`GuiMachineInfuser`), with the player inventory at (10, 70).
fn screen() -> Screen {
    Screen {
        width: 180.0,
        height: 152.0,
        slots: vec![Some([64.0, 24.0]), Some([118.0, 24.0])],
        inventory_y: 70.0,
        inventory_x: 10.0,
        own_labels: false,
        own_background: false,
        ..Screen::default()
    }
}

// A block entity Part is also the block's Part (the same type answers both), so the trait bound
// needs an implementation; every `BlockPart` method has its default here.
#[mod_api::horizon_part]
impl BlockPart for MachinePart {}

#[mod_api::horizon_part]
impl BlockEntityPart for MachinePart {
    /// The machine ticks on the server (`EntityBlock.getTicker`).
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: count an infusion down, or start one.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        infusion_tick(host, event);
    }

    /// The machine's two slots, at upstream's pixels.
    fn screen(&self, _host: &dyn Host, _event: &mut ScreenEvent<'_>) -> Option<Screen> {
        Some(screen())
    }

    /// The buffer, the block's infused level and the countdown, from the menu's data slots
    /// `[energy/32, capacity/32, infusing, infused]`.
    fn screen_draw(&self, _host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        let (energy, infused, infusing) = screen_texts(event.data);
        let origin = event.origin;
        event.draw.text(
            &energy,
            [origin[0] + 8.0, origin[1] + 6.0],
            SCREEN_GREY,
            false,
        );
        event.draw.text(
            &infused,
            [origin[0] + 8.0, origin[1] + 16.0],
            SCREEN_GREY,
            false,
        );
        if let Some(infusing) = infusing {
            event.draw.text(
                &infusing,
                [origin[0] + 8.0, origin[1] + 26.0],
                SCREEN_GREY,
                false,
            );
        }
    }
}

#[mod_api::horizon_part]
impl ItemPart for MachineItem {
    /// The item's `Infused: N%` line (`BaseBlock.intAddInformation`), once it has a level.
    fn tooltip(&self, _host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
        let level = infused_level(event.stack);
        if level == 0 {
            return;
        }
        let percent = level * 100 / MAX_INFUSE;
        event.lines.push((
            Text::literal(format!("Infused: {percent}%")),
            TOOLTIP_YELLOW,
        ));
    }

    /// A placed infused machine seeds the block it becomes (`applyImplicitComponents` →
    /// `DefaultInfusable.load`), so the new machine infuses more cheaply.
    fn on_block_placed_by(&self, host: &mut dyn ServerHost, event: &mut OnBlockPlacedByEvent<'_>) {
        let level = infused_level(event.stack);
        if level == 0 {
            return;
        }
        let pos = event.pos;
        let Some(level_ref) = placed_level(&*host, pos) else {
            return;
        };
        let Some(entity) = host.block_entity(level_ref, pos) else {
            return;
        };
        let Some(mut data) = entity.data(&*host) else {
            return;
        };
        write_int(&mut data, KEY_INFUSED, level);
        let _ = entity.replace_data(host, data);
    }
}

// ---------------------------------------------------------------------------
// The infusion tick (`MachineInfuserTileEntity.tickServer`)
// ---------------------------------------------------------------------------

/// One server tick of the machine: read its state, let [`runtime`] decide, and write back only
/// what moved (the tick answers `Changed` only then, so an idle machine never dirties the chunk).
fn infusion_tick(host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
    let mut items_changed = false;
    if event.entity.items.len() < SLOT_COUNT {
        event.entity.items.resize(SLOT_COUNT, None);
        items_changed = true;
    }
    let own_infused = read_int(&event.entity.nbt, KEY_INFUSED).clamp(0, MAX_INFUSE);
    let mut machine = Machine::new();
    machine.energy = read_int(&event.entity.nbt, KEY_ENERGY).clamp(0, CAPACITY);
    machine.infusing = read_int(&event.entity.nbt, KEY_INFUSING).clamp(0, INFUSING_TICKS);
    let before = machine;
    let has_shard = event.entity.items[SHARD_SLOT]
        .as_ref()
        .is_some_and(|stack| stack.count() > 0 && is_shard(&*host, stack));
    let target = target_level(&*host, &event.entity.items);
    let tick = machine.tick(has_shard, target, own_infused);
    if tick.consumed_shard {
        take_shard(&mut event.entity.items);
        items_changed = true;
    }
    if tick.finished {
        items_changed |= raise_target(&*host, &mut event.entity.items);
    }
    if machine != before {
        write_int(&mut event.entity.nbt, KEY_ENERGY, machine.energy);
        write_int(&mut event.entity.nbt, KEY_INFUSING, machine.infusing);
    }
    let mut data = machine.data_slots().to_vec();
    data.push(own_infused);
    let data_changed = event.entity.data != data;
    if data_changed {
        event.entity.data = data;
    }
    if machine == before && !data_changed && !items_changed {
        *event.reply = BlockEntityTickReply::Unchanged;
    }
}

/// Is the item in slot 0 a `rftoolsbase:shards` entry (`TagTools.hasTag`)?
fn is_shard(host: &dyn Host, stack: &ItemStack) -> bool {
    let Ok(tag) = Identifier::parse(SHARD_TAG) else {
        return false;
    };
    let Some(key) = host.tags().key(&tag) else {
        return false;
    };
    host.tags().is_in_tag(stack.item(), key).unwrap_or(false)
}

/// An infusable machine block item (`BaseBlock.isInfusable`), at any level: the runtime refuses
/// 256 itself. Other RFTools mods' machines are not in this port.
fn is_infusable(host: &dyn Host, stack: &ItemStack) -> bool {
    stack.count() > 0 && stack.item().id(host) == MACHINE_INFUSER
}

/// The infused level of the machine in slot 1, or `None` when the slot holds nothing this port
/// can infuse (`getStackIfInfusable`).
fn target_level(host: &dyn Host, items: &[Option<ItemStack>]) -> Option<i32> {
    let stack = items.get(TARGET_SLOT)?.as_ref()?;
    is_infusable(host, stack).then(|| infused_level(stack))
}

/// `items.getStackInSlot(0).split(1)`.
fn take_shard(items: &mut [Option<ItemStack>]) {
    let Some(Some(stack)) = items.get_mut(SHARD_SLOT) else {
        return;
    };
    let count = stack.count();
    if count > 1 {
        let _ = stack.set_count(count - 1);
    } else {
        items[SHARD_SLOT] = None;
    }
}

/// `finishInfusing`: raise the machine in slot 1 by one level, unless it left or reached 256.
fn raise_target(host: &dyn Host, items: &mut [Option<ItemStack>]) -> bool {
    let Some(stack) = items.get_mut(TARGET_SLOT).and_then(Option::as_mut) else {
        return false;
    };
    if !is_infusable(host, stack) {
        return false;
    }
    let level = infused_level(stack);
    if level >= MAX_INFUSE {
        return false;
    }
    write_infused(stack, level + 1)
}

/// The `rftoolsbase:infused` component, 0 when absent.
fn infused_level(stack: &ItemStack) -> i32 {
    match stack
        .components()
        .and_then(|patch| patch.as_nbt().get(&COMPONENT_INFUSED.into()))
    {
        Some(Tag::Int(level)) => (*level).clamp(0, MAX_INFUSE),
        _ => 0,
    }
}

/// Store an infused level on a machine stack; false when the stack refused the patch.
fn write_infused(stack: &mut ItemStack, level: i32) -> bool {
    let patch: Compound = [(COMPONENT_INFUSED.into(), Tag::Int(level))]
        .into_iter()
        .collect();
    stack.apply_components(&patch).is_ok()
}

// ---------------------------------------------------------------------------
// The commands (`/rftoolsbase info|energy|infused|level|held`)
// ---------------------------------------------------------------------------

/// The five harness controls. Upstream has none: they let the in-game check read and set a
/// machine without a GUI (precedent `/quarryplus info|energy`, `/powah info|set`).
fn commands() -> Vec<Command> {
    vec![
        Command::new("rftoolsbase info <pos:block_pos> [nonce:word]"),
        Command::op("rftoolsbase energy <pos:block_pos> <amount:integer>", 2),
        Command::op("rftoolsbase infused <pos:block_pos> <amount:integer>", 2),
        Command::op("rftoolsbase level <pos:block_pos> <amount:integer>", 2),
        Command::op("rftoolsbase held <amount:integer>", 2),
    ]
}

/// Print a line as the command's own source does (the player, the console, a command block).
fn feedback(host: &mut dyn ServerHost, line: &str) {
    let _ = host.commands().feedback(Text::literal(line));
}

/// Run a parsed `/rftoolsbase` command.
fn run_command(host: &mut dyn ServerHost, event: &mut RunModCommandEvent<'_>) -> Flow {
    let invocation = event.command;
    let literals: Vec<&str> = invocation.literals.iter().map(String::as_str).collect();
    match literals.as_slice() {
        ["rftoolsbase", "info"] => {
            let Some(CommandArgumentValue::BlockPos(pos)) = invocation.argument("pos") else {
                return Flow::Continue;
            };
            let nonce = match invocation.argument("nonce") {
                Some(CommandArgumentValue::String(nonce)) => Some(nonce.as_str()),
                _ => None,
            };
            // The readout names both slots' items and counts and the target's infused level:
            // machine contents, not public state. An operator reads any machine; a player only
            // one within ordinary interaction reach; any other non-operator source nothing.
            let (operator, reader) = reader(host, event.source);
            if !runtime::may_read(operator, reader, *pos) {
                feedback(host, "That machine is out of reach.");
                return Flow::Cancel;
            }
            let Some(level) = dimension_level(host, invocation.dimension) else {
                feedback(host, "That dimension is not open.");
                return Flow::Cancel;
            };
            let Some(entity) = machine_at(&*host, level, *pos) else {
                feedback(host, &format!("{pos:?} is not a machine infuser."));
                return Flow::Cancel;
            };
            let line = readout(&*host, level, entity, *pos, nonce);
            feedback(host, &line);
        }
        ["rftoolsbase", "energy" | "level"] => {
            let Some(CommandArgumentValue::BlockPos(pos)) = invocation.argument("pos") else {
                return Flow::Continue;
            };
            let Some(CommandArgumentValue::Integer(amount)) = invocation.argument("amount") else {
                feedback(host, "The amount must be a whole number.");
                return Flow::Cancel;
            };
            let Some(level) = dimension_level(host, invocation.dimension) else {
                feedback(host, "That dimension is not open.");
                return Flow::Cancel;
            };
            let Some(entity) = machine_at(&*host, level, *pos) else {
                feedback(host, &format!("{pos:?} is not a machine infuser."));
                return Flow::Cancel;
            };
            let (key, value) = if literals[1] == "energy" {
                (KEY_ENERGY, (*amount).clamp(0, CAPACITY))
            } else {
                (KEY_INFUSED, (*amount).clamp(0, MAX_INFUSE))
            };
            let Some(mut data) = entity.data(&*host) else {
                feedback(host, &format!("{pos:?} is not a machine infuser."));
                return Flow::Cancel;
            };
            write_int(&mut data, key, value);
            if entity.replace_data(host, data).is_err() {
                feedback(host, &format!("{pos:?} is not a machine infuser."));
                return Flow::Cancel;
            }
            feedback(host, &set_line(key, *pos, value));
        }
        ["rftoolsbase", "infused"] => {
            let Some(CommandArgumentValue::BlockPos(pos)) = invocation.argument("pos") else {
                return Flow::Continue;
            };
            let Some(CommandArgumentValue::Integer(amount)) = invocation.argument("amount") else {
                feedback(host, "The amount must be a whole number.");
                return Flow::Cancel;
            };
            let Some(level) = dimension_level(host, invocation.dimension) else {
                feedback(host, "That dimension is not open.");
                return Flow::Cancel;
            };
            let Some(entity) = machine_at(&*host, level, *pos) else {
                feedback(host, &format!("{pos:?} is not a machine infuser."));
                return Flow::Cancel;
            };
            match set_target_level(host, entity, (*amount).clamp(0, MAX_INFUSE)) {
                Some(level) => feedback(
                    host,
                    &format!(
                        "Machine Infuser at {pos:?}: the machine in slot 1 is infused {level}/{MAX_INFUSE}."
                    ),
                ),
                None => feedback(
                    host,
                    &format!("{pos:?} has no infusable machine in slot 1."),
                ),
            }
        }
        ["rftoolsbase", "held"] => {
            let CommandSource::Player { player, .. } = event.source else {
                // The held item is a player's; the console holds none.
                return Flow::Continue;
            };
            let player = *player;
            let Some(CommandArgumentValue::Integer(amount)) = invocation.argument("amount") else {
                feedback(host, "The amount must be a whole number.");
                return Flow::Cancel;
            };
            match set_held_level(host, player, (*amount).clamp(0, MAX_INFUSE)) {
                Some(level) => feedback(
                    host,
                    &format!("Machine Infuser held item: infused {level}/{MAX_INFUSE}."),
                ),
                None => feedback(host, "Hold a Machine Infuser item first."),
            }
        }
        _ => return Flow::Continue,
    }
    Flow::Cancel
}

/// What `/rftoolsbase energy|level` answers after a write.
fn set_line(key: &str, pos: [i32; 3], value: i32) -> String {
    if key == KEY_ENERGY {
        format!("Machine Infuser at {pos:?}: {value}/{CAPACITY} FE.")
    } else {
        format!("Machine Infuser at {pos:?}: block infused {value}/{MAX_INFUSE}.")
    }
}

/// The source's right to read machine contents: whether its permission reaches the operator
/// level (2, the level every other `/rftoolsbase` subcommand requires), and the player's
/// position when the source is a player.
fn reader(host: &dyn ServerHost, source: &CommandSource) -> (bool, Option<[f64; 3]>) {
    let operator = source.permission().level() >= 2;
    let player = match source {
        CommandSource::Player { player, .. } => player.entity().position(host),
        _ => None,
    };
    (operator, player)
}

/// The open level the invocation executes in (after `execute in`).
fn dimension_level(host: &mut dyn ServerHost, dimension: DimensionKey) -> Option<LevelRef> {
    host.levels()
        .list_levels()
        .into_iter()
        .find(|level| level.dimension() == dimension)
}

/// The machine's block entity at `pos`, if the block there is ours.
fn machine_at(host: &dyn Host, level: LevelRef, pos: [i32; 3]) -> Option<BlockEntityRef> {
    let block = level.block(host, pos)?;
    if block.id(host) != MACHINE_INFUSER {
        return None;
    }
    host.block_entity(level, pos)
}

/// The open level the machine at `pos` just went into, found by the block standing there (a
/// placement happens in the acting level, which the item's Part is not told directly).
fn placed_level(host: &dyn ServerHost, pos: [i32; 3]) -> Option<LevelRef> {
    host.levels().list_levels().into_iter().find(|level| {
        level
            .block(host, pos)
            .is_some_and(|b| b.id(host) == MACHINE_INFUSER)
    })
}

/// The machine's state, as `/rftoolsbase info` prints it, with the harness's nonce when given.
fn readout(
    host: &dyn Host,
    level: LevelRef,
    entity: BlockEntityRef,
    pos: [i32; 3],
    nonce: Option<&str>,
) -> String {
    let Some(data) = entity.data(host) else {
        return format!("{pos:?} is not a mod block entity.");
    };
    let energy = read_int(&data, KEY_ENERGY).clamp(0, CAPACITY);
    let infusing = read_int(&data, KEY_INFUSING).clamp(0, INFUSING_TICKS);
    let infused = read_int(&data, KEY_INFUSED).clamp(0, MAX_INFUSE);
    let shard = slot_text(
        host,
        entity.item_slot(host, SHARD_SLOT as u32).ok().flatten(),
    );
    let target = match entity.item_slot(host, TARGET_SLOT as u32).ok().flatten() {
        Some(stack) => format!(
            "{} x{} infused {}",
            stack.item().id(host),
            stack.count(),
            infused_level(&stack)
        ),
        None => "empty".to_owned(),
    };
    let time = level.game_time(host).unwrap_or(0);
    readout_line(
        pos,
        (energy, infusing, infused),
        &shard,
        &target,
        time,
        nonce,
    )
}

/// The `info` line's exact shape, kept separate so it is testable without a game. `state` is the
/// machine's `(energy, infusing, infused)`.
fn readout_line(
    pos: [i32; 3],
    state: (i32, i32, i32),
    shard: &str,
    target: &str,
    time: i64,
    nonce: Option<&str>,
) -> String {
    let (energy, infusing, infused) = state;
    let suffix = nonce.map_or(String::new(), |nonce| format!(" nonce={nonce}"));
    format!(
        "Machine Infuser at {pos:?}: {energy}/{CAPACITY} FE, infusing {infusing}/{INFUSING_TICKS}, block infused {infused}/{MAX_INFUSE}, shard {shard}, target {target}, t={time}{suffix}"
    )
}

/// One slot as `item xN`, or `empty`.
fn slot_text(host: &dyn Host, stack: Option<ItemStack>) -> String {
    match stack {
        Some(stack) => format!("{} x{}", stack.item().id(host), stack.count()),
        None => "empty".to_owned(),
    }
}

/// Set the machine in the machine's slot 1 to `target` infused levels.
fn set_target_level(host: &mut dyn ServerHost, entity: BlockEntityRef, target: i32) -> Option<i32> {
    let stack = entity
        .item_slot(&*host, TARGET_SLOT as u32)
        .ok()
        .flatten()?;
    if !is_infusable(&*host, &stack) {
        return None;
    }
    let mut edited = stack;
    if !write_infused(&mut edited, target) {
        return None;
    }
    let written = host
        .transaction(|host, tx| entity.set_item_slot(host, tx, TARGET_SLOT as u32, Some(edited)))
        .ok()?;
    written.then_some(target)
}

/// Set the machine item the acting player holds to `target` infused levels (the harness's way
/// to make an infused item).
fn set_held_level(host: &mut dyn ServerHost, player: PlayerRef, target: i32) -> Option<i32> {
    // The selected slot's stack, written back through `set_inventory_slot`: the inventory is what
    // the placement reads, so the level has to be on the real stack (an edit of a hand's copy
    // answered success and left the item unchanged at the 0.2.1 e2e).
    let slot = u32::try_from(player.selected_slot(&*host)?).ok()?;
    let mut stack = player
        .inventory_slots(&*host)?
        .into_iter()
        .find(|(at, _)| *at == slot)
        .map(|(_, stack)| stack)?;
    if !is_infusable(&*host, &stack) {
        return None;
    }
    if !write_infused(&mut stack, target) {
        return None;
    }
    player.set_inventory_slot(host, slot, Some(stack)).ok()?;
    Some(target)
}

// ---------------------------------------------------------------------------
// Saved-compound reads and the menu's data slots
// ---------------------------------------------------------------------------

/// One integer of the saved compound, by its Java key. `Compound` is a map keyed by Java's
/// lossless UTF-16 strings, which the SDK does not export by name, so the lookup compares the
/// stored units.
fn read_int(nbt: &Compound, name: &str) -> i32 {
    let units: Vec<u16> = name.encode_utf16().collect();
    match nbt.iter().find(|(key, _)| key.0 == units) {
        Some((_, Tag::Int(value))) => *value,
        _ => 0,
    }
}

fn write_int(nbt: &mut Compound, name: &str, value: i32) {
    nbt.insert(name.into(), Tag::Int(value));
}

/// The screen's three readouts from the menu's data slots
/// `[energy/32, capacity/32, infusing, infused]`.
fn screen_texts(data: &[i32]) -> (String, String, Option<String>) {
    let energy = data.first().copied().unwrap_or(0) * 32;
    let capacity = data.get(1).copied().unwrap_or(0) * 32;
    let infusing = data.get(2).copied().unwrap_or(0);
    let infused = data.get(3).copied().unwrap_or(0);
    (
        format!("{energy}/{capacity} FE"),
        format!("infused {infused}/{MAX_INFUSE}"),
        (infusing > 0).then(|| format!("infusing {infusing}")),
    )
}

// ---------------------------------------------------------------------------
// The mod
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the machine's block entity type, its two slots, its FE buffer and its item Part.
    fn register(&self, r: &mut dyn Registrar, _context: &RegistrationContext) {
        let machine = r.block_key(MACHINE_INFUSER);
        let kind = r
            .block_entity_type(MACHINE_TYPE)
            .blocks(&[MACHINE_INFUSER])
            .build();
        let _ = r.block_entity_part(&kind, MachinePart);
        r.block_container(&machine, container());
        r.block_energy(&machine, buffer());
        let item = r.item_key(MACHINE_INFUSER);
        r.item_part(item, MachineItem);
    }

    /// Register `rftoolsbase:infused`, the item component the machine's level travels in. The
    /// content is refused at load when this id is unknown, so it must be pushed here.
    fn component_types(&self, _context: &RegistrationContext, event: &mut ComponentTypesEvent<'_>) {
        event.types.push(COMPONENT_INFUSED.to_owned());
    }

    /// `/rftoolsbase info|energy|infused|level|held`: the harness controls.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event.commands.extend(commands());
    }

    /// The bodies of those commands, on the already-parsed invocation.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        run_command(host, event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_container_is_two_slots_on_every_face_with_slot_one_out() {
        let container = container();
        assert_eq!(container.slots, SLOT_COUNT);
        assert_eq!(
            container.faces,
            array::from_fn::<Vec<usize>, 6, _>(|_| vec![SHARD_SLOT, TARGET_SLOT])
        );
        assert_eq!(container.inputs, vec![SHARD_SLOT, TARGET_SLOT]);
        assert_eq!(container.outputs, vec![TARGET_SLOT]);
        assert_eq!(container.menu.as_deref(), Some(MENU));
    }

    #[test]
    fn the_buffer_takes_energy_on_every_face_and_gives_none() {
        let energy = buffer();
        assert_eq!(energy.capacity, CAPACITY);
        assert_eq!(energy.max_insert, MAX_INSERT);
        assert_eq!(energy.max_extract, 0);
        assert_eq!(energy.insert_faces, [true; 6]);
        assert_eq!(energy.extract_faces, [false; 6]);
    }

    #[test]
    fn the_screen_keeps_upstreams_slots_and_panel() {
        let screen = screen();
        assert_eq!([screen.width, screen.height], [180.0, 152.0]);
        assert_eq!(screen.slots, vec![Some([64.0, 24.0]), Some([118.0, 24.0])]);
        assert_eq!([screen.inventory_x, screen.inventory_y], [10.0, 70.0]);
        assert!(!screen.own_labels && !screen.own_background);
    }

    #[test]
    fn the_saved_ints_round_trip_and_an_absent_key_reads_zero() {
        let mut nbt = Compound::new();
        write_int(&mut nbt, KEY_ENERGY, 60_000);
        write_int(&mut nbt, KEY_INFUSED, 255);
        assert_eq!(read_int(&nbt, KEY_ENERGY), 60_000);
        assert_eq!(read_int(&nbt, KEY_INFUSED), 255);
        assert_eq!(read_int(&nbt, KEY_INFUSING), 0);
    }

    #[test]
    fn the_info_line_names_the_machine_and_its_numbers() {
        let line = readout_line(
            [202, 101, 0],
            (299, 0, 255),
            "empty",
            "rftoolsbase:machine_infuser x1 infused 1",
            460,
            Some("5"),
        );
        assert_eq!(
            line,
            "Machine Infuser at [202, 101, 0]: 299/60000 FE, infusing 0/5, block infused 255/256, shard empty, target rftoolsbase:machine_infuser x1 infused 1, t=460 nonce=5"
        );
    }

    #[test]
    fn the_screen_reads_its_data_slots_as_thirty_two_fe_units() {
        let (energy, infused, infusing) = screen_texts(&[1875, 1875, 5, 100]);
        assert_eq!(energy, "60000/60000 FE");
        assert_eq!(infused, "infused 100/256");
        assert_eq!(infusing.as_deref(), Some("infusing 5"));
        let (energy, _, infusing) = screen_texts(&[]);
        assert_eq!(energy, "0/0 FE");
        assert_eq!(infusing, None);
    }

    #[test]
    fn the_five_harness_commands_keep_their_usage_and_levels() {
        let commands = commands();
        assert_eq!(commands.len(), 5);
        assert_eq!(
            commands[0].usage,
            "rftoolsbase info <pos:block_pos> [nonce:word]"
        );
        assert_eq!(commands[0].level, 0);
        assert!(commands[1..].iter().all(|command| command.level == 2));
    }
}
