//! Starling quality-of-life tweaks: one place for every switch and number.
//!
//! Flip a `bool` to `false` to turn a feature off, or change a number, then
//! rebuild. Nothing here reads a config file, so the tweaks always behave the
//! same on every phone.

/// Largest stack of any item that stacks at all (vanilla: 64, some items 16).
/// Items that do not stack in vanilla (tools, armor, potions...) stay at 1.
pub const MAX_STACK: u32 = 1000;

/// Map an item's vanilla stack size to the one the game uses.
pub const fn stack_cap(vanilla: u32) -> u32 {
    if vanilla > 1 { MAX_STACK } else { vanilla }
}

/// Mining an ore breaks the whole connected vein. Crouch to mine one block.
pub const VEIN_MINER: bool = true;
/// Most blocks one vein break removes (besides the one you hit).
pub const VEIN_MAX_BLOCKS: usize = 64;

/// Breaking a log fells the whole tree, leaves included. Crouch to chop one log.
pub const TREE_FELLER: bool = true;
/// Most logs one felling removes (besides the one you hit).
pub const TREE_MAX_LOGS: usize = 256;
/// Most leaf blocks one felling clears.
pub const TREE_MAX_LEAVES: usize = 512;

/// Blocks you break drop straight into your inventory.
pub const AUTO_PICKUP: bool = true;

/// Ticks to wait between two placements while the use button is held (vanilla 4).
pub const PLACE_DELAY_TICKS: u32 = 0;

/// Ticks to wait after breaking a block before the next one starts (vanilla 5).
pub const BREAK_DELAY_TICKS: u32 = 0;
/// Multiplier on survival mining progress per tick (vanilla 1.0).
pub const BREAK_SPEED_MULTIPLIER: f32 = 5.0;

/// Creepers never explode.
pub const CREEPERS_DONT_EXPLODE: bool = true;

/// While it rains, random ticks run this many times as often, which is what
/// makes crops and saplings grow (vanilla 1). Needs `random_tick_speed` > 0.
pub const RAIN_RANDOM_TICK_MULTIPLIER: i32 = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_stackable_items_get_the_big_stack() {
        assert_eq!(stack_cap(64), MAX_STACK);
        assert_eq!(stack_cap(16), MAX_STACK);
        assert_eq!(stack_cap(1), 1);
    }
}
