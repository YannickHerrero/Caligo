# Enemies

The bestiary. Enemies are spawned in combat encounters along the map.

Source of truth: [`src/fight/enemy.rs`](../../src/fight/enemy.rs).

## Standard

| Enemy | HP | Notes |
|---|---:|---|
| Slime | 30 | The starter foe. Slow, no special abilities. Sprite is a 4-line `(o o)` blob. |

## Bosses

*None implemented yet.* The roadmap calls for one boss per floor at the end of the branching map; bosses will live alongside standard enemies in the bestiary once they exist.

## Roadmap

- Per-element enemy types so elemental matchups matter (e.g. Lava Crab, Frost Slime).
- Movesets: each enemy will pull from the [attack library](attacks.md) rather than dealing flat damage.
- Boss tier with multi-phase HP bars and unique sprites.
