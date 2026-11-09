# Starling quality-of-life tweaks

Every switch and number lives in `crates/horizon-items/src/qol.rs`. Change a
value, rebuild, done. There is no config file.

| Feature | What it does | Switch |
|---|---|---|
| Stack 1000 | Every item that stacks in vanilla stacks to 1000, in the inventory, chests, hoppers and other containers. Tools, armor and potions stay at 1. | `MAX_STACK` |
| Vein miner | Mining an ore (survival, with a tool that can harvest it) breaks the whole connected vein. Stone and deepslate ore count as one vein. | `VEIN_MINER`, `VEIN_MAX_BLOCKS` |
| Tree feller | Breaking a log of a tree with leaves fells the whole tree and clears its leaves (their sapling/apple drops stay, the tool is not worn by leaves). Logs with no leaves on them are left alone. | `TREE_FELLER`, `TREE_MAX_LOGS`, `TREE_MAX_LEAVES` |
| Auto pickup | What a block you break drops goes straight into your inventory; only what does not fit pops out. | `AUTO_PICKUP` |
| Fast place | Holding the use button places a block every tick instead of every 4th. | `PLACE_DELAY_TICKS` |
| Fast break | No wait between blocks and 5x mining progress. | `BREAK_DELAY_TICKS`, `BREAK_SPEED_MULTIPLIER` |
| Creepers | They never light their fuse, so they never explode. | `CREEPERS_DONT_EXPLODE` |
| Rain growth | Random ticks (what grows crops and saplings) run 3x as often while it rains. Other random-tick things speed up too (grass spread, leaf decay). | `RAIN_RANDOM_TICK_MULTIPLIER` |

Crouch while mining to break just one block (vein miner and tree feller are
survival-only and ignore creative mode).

Not covered: a LAN guest's own breaks, mob drops and chest-loot are not
auto-picked, and a vanilla client joining your world cannot show counts above 99.
