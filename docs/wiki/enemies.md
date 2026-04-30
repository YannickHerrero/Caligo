# Enemies

The bestiary. Each enemy has a fixed stat block; difficulty scales by *which* enemy a floor spawns rather than by an explicit level number.

Source of truth: [`src/data/enemies.rs`](../../src/data/enemies.rs) for entries, [`src/fight/enemy.rs`](../../src/fight/enemy.rs) for the struct, [`src/fight/attack.rs`](../../src/fight/attack.rs) for the type chart.

> **In-game preview:** open the **Catalogue** screen and `Tab` over to *Bestiary* to browse every enemy with their sprite, types, weaknesses, and moveset.

## Enemy data model

```rust
struct Enemy {
    name, description,           // identity + flavor
    primary_type, secondary_type, // 1 or 2 Pokémon-style types
    hp, max_hp, speed,            // HP + turn-order
    moveset: Vec<&'static str>,   // attack names from the attack library
    sprite, color,                // ASCII art
    is_boss,                      // flag for boss-tier encounters
}
```

- **No level field.** Each kind has fixed stats; the dungeon controls difficulty by spawning tougher kinds on deeper floors.
- **Variants are distinct kinds.** Slime, Fire Slime, and Frost Slime each have their own entry, sprite, types, and moves.
- **Movesets reuse the player attack library** by name — the same `Attack` definitions that power `Pinch` or `Inferno` drive enemy turns.
- **Bosses share the struct.** The `is_boss` flag gates spawn rules and adds a ★ marker in the catalogue; otherwise the data layout is identical.

## Type effectiveness

Eight types: Normal, Fire, Water, Ice, Electric, Ground, Flying, Psychic. Multipliers are 2x, 1x, or 0.5x — Pokémon-shaped, with 0x immunities collapsed to 0.5x to avoid hard locks.

| Atk \ Def | Normal | Fire | Water | Ice | Electric | Ground | Flying | Psychic |
|---|---|---|---|---|---|---|---|---|
| **Normal** | 1× | 1× | 1× | 1× | 1× | 1× | 1× | 1× |
| **Fire** | 1× | 0.5× | 0.5× | **2×** | 1× | 1× | 1× | 1× |
| **Water** | 1× | **2×** | 0.5× | 1× | 1× | **2×** | 1× | 1× |
| **Ice** | 1× | 0.5× | 0.5× | 0.5× | 1× | **2×** | **2×** | 1× |
| **Electric** | 1× | 1× | **2×** | 1× | 0.5× | 0.5× | **2×** | 1× |
| **Ground** | 1× | **2×** | 1× | 1× | **2×** | 1× | 0.5× | 1× |
| **Flying** | 1× | 1× | 1× | 1× | 0.5× | 1× | 1× | 1× |
| **Psychic** | 1× | 1× | 1× | 1× | 1× | 1× | 1× | 0.5× |

Dual-type defenders multiply the per-type multipliers — a Water/Ground enemy hit by Ice gets `2× × 2× = 4×`.

## Roster

| Enemy | Type(s) | HP | Speed | Moveset |
|---|---|---:|---:|---|
| Slime | Water | 30 | 12 | Splash, Bubble |
| Fire Slime | Fire | 28 | 14 | Ember, Cinder Spit |
| Frost Slime | Ice | 32 | 8 | Frostbite, Ice Shard |
| Sandling | Ground | 40 | 6 | Granite Shell, Sandstorm, Stone Slam |
| ★ Crab King | Water / Ground | 120 | 10 | Tidal Slam, Stone Slam, Boulder Press, Tsunami |

★ = boss.

## Roadmap

- Spawn tables: which enemy kinds appear on which floor depths.
- Status effects (burn, freeze, poison, etc.) — currently out of scope.
- Multi-phase bosses (Crab King phase 2 with a different sprite/moveset).
- Loot drops: gold, items, attack stones.
- Per-enemy AI weights for move selection.
