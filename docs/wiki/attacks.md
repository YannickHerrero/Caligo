# Attacks

The full attack pool, grouped by element. The player begins with **Pinch**, **Bubble**, **Snip**, and **Cosmic Orb** equipped; the remaining attacks are unlocked through Stones found during a run.

Source of truth: [`src/data/attacks.rs`](../../src/data/attacks.rs).

## Animation kinds

| Kind | Description |
|---|---|
| `Dash` | Crab slides in, strikes, slides back. Typically physical and cheap. |
| `Jump` | Crab arcs onto the target. Heavier hits, often higher cost. |
| `Throw(kind)` | Crab launches a projectile of the given kind in an arc. |

## Projectile kinds

| Kind | Visual | Color |
|---|---|---|
| `Water` | 1×1 droplet | Blue |
| `Fire` | 2×2 flame | Orange |
| `Electric` | 1×3 bolt | Yellow |
| `EnergyBall` | 3×3 orb | Purple |

## Neutral

Physical, mostly cheap. Reliable when mana is low.

| Name | Animation | Damage | Mana |
|---|---|---:|---:|
| Pinch ⭐ | Dash | 5 | 0 |
| Scuttle Strike | Dash | 4 | 0 |
| Headbutt | Dash | 6 | 0 |
| Tail Whip | Dash | 5 | 1 |
| Bite | Dash | 7 | 1 |
| Snip ⭐ | Jump | 8 | 2 |
| Shell Bash | Dash | 9 | 3 |
| Claw Crush | Jump | 12 | 4 |
| Double Snip | Jump | 14 | 6 |
| Final Pinch | Jump | 18 | 7 |

## Fire

Aggressive, projectile-heavy. Tends toward higher cost.

| Name | Animation | Damage | Mana |
|---|---|---:|---:|
| Ember | Throw(Fire) | 6 | 2 |
| Cinder Spit | Throw(Fire) | 7 | 3 |
| Heatwave | Throw(Fire) | 8 | 4 |
| Flame Dash | Dash | 9 | 4 |
| Fireball | Throw(Fire) | 11 | 5 |
| Sunflare | Throw(Fire) | 13 | 6 |
| Lava Lob | Throw(Fire) | 14 | 6 |
| Pyre Charge | Dash | 15 | 7 |
| Magma Crush | Jump | 17 | 8 |
| Inferno | Throw(Fire) | 21 | 10 |

## Water

Balanced. Strong efficiency at low cost; ramps to a top-tier finisher.

| Name | Animation | Damage | Mana |
|---|---|---:|---:|
| Splash | Throw(Water) | 4 | 1 |
| Bubble ⭐ | Throw(Water) | 7 | 3 |
| Frostbite | Dash | 8 | 4 |
| Ice Shard | Throw(Water) | 9 | 4 |
| Riptide | Dash | 10 | 5 |
| Whirlpool | Throw(Water) | 12 | 6 |
| Tidal Slam | Jump | 13 | 5 |
| Geyser | Jump | 15 | 7 |
| Hailstorm | Throw(Water) | 17 | 9 |
| Tsunami | Throw(Water) | 22 | 12 |

## Earth

Heavy melee. Few projectiles; biggest single-hit numbers in the game.

| Name | Animation | Damage | Mana |
|---|---|---:|---:|
| Granite Shell | Dash | 6 | 1 |
| Sandstorm | Dash | 7 | 3 |
| Quake Step | Dash | 9 | 4 |
| Stone Slam | Jump | 10 | 4 |
| Iron Pinch | Dash | 11 | 4 |
| Crystal Spike | Jump | 13 | 6 |
| Rockfall | Jump | 14 | 6 |
| Boulder Press | Jump | 16 | 8 |
| Earthquake | Jump | 18 | 9 |
| Tectonic Crush | Jump | 24 | 12 |

## Air

Lightning and cosmic. Heaviest mana sinks; biggest projectiles.

| Name | Animation | Damage | Mana |
|---|---|---:|---:|
| Spark | Throw(Electric) | 5 | 2 |
| Gust | Jump | 6 | 2 |
| Static Charge | Dash | 7 | 3 |
| Thunderclap | Throw(Electric) | 9 | 4 |
| Lightning Bolt | Throw(Electric) | 12 | 6 |
| Cosmic Orb ⭐ | Throw(EnergyBall) | 14 | 8 |
| Tornado | Jump | 14 | 7 |
| Storm Strike | Throw(Electric) | 16 | 8 |
| Star Lance | Throw(EnergyBall) | 19 | 10 |
| Sky Splitter | Throw(Electric) | 22 | 11 |

⭐ = starter attack, equipped from the start of a run.

## Balance notes

- **Free attacks (cost 0)** sit in the 4–6 damage range — meant as a fallback when out of mana.
- **Cheap (1–3 mana)** ranges 5–9 damage; the most efficient damage-per-mana lives here.
- **Mid (4–7 mana)** ranges 8–15 damage; the workhorse band.
- **Heavy (8–10 mana)** ranges 14–21 damage; commits a turn to a big hit.
- **Ultimate (11–12 mana)** ranges 22–24 damage; one-per-fight finishers.
- Within each band, *efficient* picks (high damage-for-cost) are typically `Dash` or short `Throw`s; *flashy* picks (lower efficiency) trade numbers for animation impact and will pick up status effects in future updates.
