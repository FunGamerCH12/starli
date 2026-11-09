//! Starling quality-of-life features that live in the game loop: the vein
//! miner, the tree feller and auto pickup. Switches and limits are in
//! `horizon_items::qol`.
//!
//! A chain is planned before the block you hit is gone (so the neighbours can
//! still be read), then carried out after that first break went through.
use super::Game;
use horizon_client::{Result, items::ItemStack};
use horizon_core::position::BlockPos;
use horizon_items::qol;
use std::collections::{HashSet, VecDeque};
use winit::keyboard::KeyCode;

/// What one player break sets in motion.
pub(super) struct Chain {
    origin: [i32; 3],
    /// The block that was hit, to tell later whether its break went through.
    name: String,
    kind: Kind,
}

enum Kind {
    /// The rest of the ore vein; `family` is what counts as the same ore.
    Vein {
        family: String,
        blocks: Vec<[i32; 3]>,
    },
    /// The rest of the tree: its other logs, then its leaves.
    Tree {
        logs: Vec<[i32; 3]>,
        leaves: Vec<[i32; 3]>,
    },
}

/// The ore a block is, with `deepslate_` and the namespace left off, so that
/// stone and deepslate coal ore are one vein. `None` for anything not an ore.
fn vein_family(name: &str) -> Option<&str> {
    if name == "minecraft:ancient_debris" {
        return Some("ancient_debris");
    }
    if !name.ends_with("_ore") {
        return None;
    }
    let bare = name.strip_prefix("minecraft:").unwrap_or(name);
    Some(bare.strip_prefix("deepslate_").unwrap_or(bare))
}

/// A log of an overworld tree (not a mushroom stem, not a nether stem).
fn is_tree_log(name: &str) -> bool {
    (name.ends_with("_log") || name.ends_with("_wood")) && !name.contains("mushroom")
}

fn is_leaf(name: &str) -> bool {
    name.ends_with("_leaves")
}

/// Breadth-first search outward from `start`. A neighbour joins the result
/// when `matches` accepts it, it lies within `radius` blocks of `center` on
/// every axis, and it is at most `max_depth` steps from `start`. `start`
/// itself is never part of the result. Results come nearest first.
fn flood(
    start: &[[i32; 3]],
    center: [i32; 3],
    diagonal: bool,
    radius: i32,
    max_depth: usize,
    limit: usize,
    matches: &dyn Fn([i32; 3]) -> bool,
) -> Vec<[i32; 3]> {
    let mut offsets = Vec::new();
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let steps = (dx != 0) as i32 + (dy != 0) as i32 + (dz != 0) as i32;
                if steps == 0 || (!diagonal && steps > 1) {
                    continue;
                }
                offsets.push([dx, dy, dz]);
            }
        }
    }
    let mut seen: HashSet<[i32; 3]> = start.iter().copied().collect();
    let mut queue: VecDeque<([i32; 3], usize)> = start.iter().map(|&p| (p, 0)).collect();
    let mut found = Vec::new();
    while let Some((at, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        for o in &offsets {
            let next = [at[0] + o[0], at[1] + o[1], at[2] + o[2]];
            if (0..3).any(|i| (next[i] - center[i]).abs() > radius) || !seen.insert(next) {
                continue;
            }
            if !matches(next) {
                continue;
            }
            found.push(next);
            if found.len() >= limit {
                return found;
            }
            queue.push_back((next, depth + 1));
        }
    }
    found
}

impl Game {
    fn qol_name(&self, p: [i32; 3]) -> &str {
        let state = self.world.block(p[0], p[1], p[2]);
        &self.res.facts().get(state).name
    }

    /// Plans the vein or the tree that breaking `origin` takes with it, if
    /// any. Only survival players get one, and not while crouching.
    pub(super) fn qol_chain_start(&self, origin: [i32; 3]) -> Option<Chain> {
        if self.creative
            || self.lan_acting.is_some()
            || self.player.is_none()
            || self.keys.contains(&KeyCode::ShiftLeft)
        {
            return None;
        }
        let name = self.qol_name(origin).to_owned();
        if qol::VEIN_MINER
            && let Some(family) = vein_family(&name)
        {
            // A pickaxe that cannot harvest the ore would only waste the others.
            if !self.items().correct_for_drops(self.held(), &name) {
                return None;
            }
            let blocks = flood(
                &[origin],
                origin,
                true,
                10,
                usize::MAX,
                qol::VEIN_MAX_BLOCKS,
                &|p| vein_family(self.qol_name(p)) == Some(family),
            );
            if blocks.is_empty() {
                return None;
            }
            let family = family.to_owned();
            return Some(Chain {
                origin,
                name,
                kind: Kind::Vein { family, blocks },
            });
        }
        if qol::TREE_FELLER && is_tree_log(&name) {
            let logs = flood(
                &[origin],
                origin,
                true,
                12,
                usize::MAX,
                qol::TREE_MAX_LOGS,
                &|p| is_tree_log(self.qol_name(p)),
            );
            let mut roots = logs.clone();
            roots.push(origin);
            // Leaves hang within seven steps of a log (the `distance` property).
            let leaves = flood(
                &roots,
                origin,
                false,
                14,
                7,
                qol::TREE_MAX_LEAVES,
                &|p| is_leaf(self.qol_name(p)),
            );
            // Logs with no leaf on them are somebody's building, not a tree.
            if leaves.is_empty() {
                return None;
            }
            return Some(Chain {
                origin,
                name,
                kind: Kind::Tree { logs, leaves },
            });
        }
        None
    }

    /// Carries out a planned chain after the first block was broken.
    pub(super) fn qol_chain_finish(&mut self, chain: Option<Chain>) -> Result<()> {
        let Some(chain) = chain else {
            return Ok(());
        };
        // The first break may have been refused (a mod, a rule): then no chain.
        if self.qol_name(chain.origin) == chain.name {
            return Ok(());
        }
        match chain.kind {
            Kind::Vein { family, blocks } => {
                for p in blocks {
                    let name = self.qol_name(p).to_owned();
                    if vein_family(&name) != Some(family.as_str()) {
                        continue;
                    }
                    // The pickaxe broke, or changed: stop here.
                    if !self.items().correct_for_drops(self.held(), &name) {
                        break;
                    }
                    self.break_block(p)?;
                }
            }
            Kind::Tree { mut logs, leaves } => {
                // Top first, as a felled tree comes down.
                logs.sort_by_key(|p| std::cmp::Reverse(p[1]));
                for p in logs {
                    if is_tree_log(self.qol_name(p)) {
                        self.break_block(p)?;
                    }
                }
                for p in leaves {
                    if is_leaf(self.qol_name(p)) {
                        self.qol_clear_leaf(p);
                    }
                }
                self.process_level_events()?;
            }
        }
        Ok(())
    }

    /// Removes one leaf block with its usual drops (saplings, sticks, apples)
    /// and without wearing the tool, which a whole tree of leaves would ruin.
    fn qol_clear_leaf(&mut self, pos: [i32; 3]) {
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let removed = self.with_level(|b, level, ticks| {
            b.player_destroy_block(
                level,
                ticks,
                BlockPos::new(pos[0], pos[1], pos[2]),
                false,
                true,
            )
        });
        if !removed {
            return;
        }
        let drops = self.block_loot_at(pos, state, None);
        for d in drops {
            if &*d.item != "minecraft:air" {
                self.qol_collect_drop(pos, d);
            }
        }
    }

    /// A block's drop, straight into the player's inventory; what does not
    /// fit pops out of the block as usual. Other actors (a LAN guest) and
    /// the switch being off keep the usual popped item.
    pub(super) fn qol_collect_drop(&mut self, pos: [i32; 3], mut stack: ItemStack) {
        if !qol::AUTO_PICKUP || self.lan_acting.is_some() || self.player.is_none() {
            self.pop_resource(pos, stack);
            return;
        }
        self.pending.attach(pos, &mut stack);
        if !self.ui.rules.bool("block_drops") {
            return;
        }
        if let Some(left) = self.give(stack) {
            let at = [
                f64::from(pos[0]) + 0.5,
                f64::from(pos[1]) + 0.5,
                f64::from(pos[2]) + 0.5,
            ];
            self.spawn_item(left, at, [0.0, 0.2, 0.0], 10);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deepslate_and_stone_ore_are_one_vein() {
        assert_eq!(
            vein_family("minecraft:deepslate_iron_ore"),
            vein_family("minecraft:iron_ore")
        );
        assert_ne!(
            vein_family("minecraft:iron_ore"),
            vein_family("minecraft:gold_ore")
        );
        assert_ne!(
            vein_family("minecraft:nether_gold_ore"),
            vein_family("minecraft:gold_ore")
        );
        assert_eq!(vein_family("minecraft:stone"), None);
        assert_eq!(vein_family("minecraft:raw_iron_block"), None);
        assert!(vein_family("minecraft:ancient_debris").is_some());
    }

    #[test]
    fn mushroom_stems_and_planks_are_not_tree_logs() {
        assert!(is_tree_log("minecraft:oak_log"));
        assert!(is_tree_log("minecraft:stripped_birch_wood"));
        assert!(!is_tree_log("minecraft:mushroom_stem"));
        assert!(!is_tree_log("minecraft:oak_planks"));
        assert!(is_leaf("minecraft:azalea_leaves"));
    }

    /// A 1x1 vertical line of blocks from y=0 to y=4, plus a stray block that
    /// touches it only diagonally and one that touches nothing.
    fn line_world(p: [i32; 3]) -> bool {
        (p[0] == 0 && p[2] == 0 && (0..5).contains(&p[1])) || p == [1, 5, 1] || p == [9, 9, 9]
    }

    #[test]
    fn flood_follows_neighbours_including_diagonals() {
        let found = flood(&[[0, 0, 0]], [0, 0, 0], true, 10, usize::MAX, 100, &line_world);
        let mut want: Vec<_> = (1..5).map(|y| [0, y, 0]).collect();
        want.push([1, 5, 1]);
        let mut got = found.clone();
        got.sort();
        want.sort();
        assert_eq!(got, want);
    }

    #[test]
    fn flood_without_diagonals_stops_at_the_stray_block() {
        let found = flood(&[[0, 0, 0]], [0, 0, 0], false, 10, usize::MAX, 100, &line_world);
        assert_eq!(found.len(), 4);
        assert!(!found.contains(&[1, 5, 1]));
    }

    #[test]
    fn flood_respects_limit_radius_and_depth() {
        let all = |p: [i32; 3]| p[0] == 0 && p[2] == 0 && (0..50).contains(&p[1]);
        assert_eq!(flood(&[[0, 0, 0]], [0, 0, 0], true, 100, usize::MAX, 7, &all).len(), 7);
        assert_eq!(flood(&[[0, 0, 0]], [0, 0, 0], true, 3, usize::MAX, 100, &all).len(), 3);
        assert_eq!(flood(&[[0, 0, 0]], [0, 0, 0], true, 100, 5, 100, &all).len(), 5);
    }

    #[test]
    fn flood_from_several_roots_measures_depth_from_the_nearest() {
        let all = |p: [i32; 3]| p[0] == 0 && p[2] == 0 && (0..=20).contains(&p[1]);
        // Roots at y=0 and y=10; two steps reaches y=1..=2 and y=8..=12, minus the roots.
        let found = flood(&[[0, 0, 0], [0, 10, 0]], [0, 10, 0], true, 100, 2, 100, &all);
        let mut ys: Vec<i32> = found.iter().map(|p| p[1]).collect();
        ys.sort();
        assert_eq!(ys, vec![1, 2, 8, 9, 11, 12]);
    }
}
