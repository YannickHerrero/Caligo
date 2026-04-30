# Attacks

The full attack pool, grouped by type. Both offensive and support moves live in the same Attack menu — the `Effect` column tells you what each one does.

The player begins with **Pinch**, **Bubble**, **Snip**, and **Cosmic Orb** equipped; the remaining attacks are unlocked through Stones found during a run.

Source of truth: [`src/data/attacks.rs`](../../src/data/attacks.rs).

## Types

Caligo borrows Pokémon-style type names. The current set is **Normal, Fire, Water, Ice, Electric, Ground, Flying, Psychic** — eight types. A type-effectiveness chart isn't implemented yet; types currently just color the attacks and pick the trail/impact particle.

## Animation kinds

| Kind | Description |
|---|---|
| `Dash` | Crab slides in, strikes, slides back. Typically physical and cheap. |
| `Jump` | Crab arcs onto the target. Heavier hits, often higher cost. |
| `Throw(kind)` | Crab launches a projectile of the given kind in an arc. |
| `SelfCast(particle)` | Crab hops in place while particles drift outward and upward around it. Used for self-targeting healing and buffs. |

## Projectile kinds

Each kind has three sizes. The size used for a `Throw` attack is derived from the attack's damage:

| Damage | Size |
|---|---|
| 0 – 8 | Small |
| 9 – 16 | Medium |
| 17+ | Large |

| Kind | Small | Medium | Large | Color |
|---|---|---|---|---|
| `Water` | 1×1 | 3×2 | 5×3 | Blue |
| `Fire` | 2×2 | 3×3 | 4×4 | Orange |
| `Electric` | 1×3 | 2×4 | 3×5 | Yellow |
| `EnergyBall` | 3×2 | 5×3 | 7×5 | Purple |

Sprite dimensions are *width × height* in characters. Round projectiles (Water and EnergyBall) are deliberately wider than tall because terminal cells render at roughly a 2:1 height:width ratio — a square sprite reads as a vertical bar on screen, so the round shapes are flattened to look round once rendered. Fire and Electric projectiles are kept tall on purpose (flames rise, lightning is a vertical bolt).

## Particle kinds

Particles are used in three places:

1. **`SelfCast` aura** — drifts outward and upward around the stationary crab during heals and buffs. Picked per-attack to match the effect.
2. **`Jump` / `Dash` elemental trails** — emitted behind the moving crab during the *outbound* leg of the animation only. The trail vanishes the moment the crab reaches the target. No trail for Normal attacks.
3. **Impact marks** — every damage attack (`Jump`, `Dash`, or `Throw`) leaves a small burst of particles at the target when the hit lands, lingering for about a second. The mark uses the attack's type, with a gray fallback for Normal.

| Kind | Glyph | Color | Used by |
|---|---|---|---|
| `Hearts` | ♥ | Pink | All `Heal` attacks (Salve, Mend, First Aid, Greater Mend) |
| `Triangles` | ▲ | Red | `AttackUp` buffs (Sharpen) |
| `Circles` | ● | Blue | `DefenseUp` buffs (Carapace) |
| `FireSpark` | * | Orange | Fire trail and impact |
| `WaterDroplet` | . | Blue | Water trail and impact |
| `IceShard` | + | Cyan | Ice trail and impact |
| `ElectricSpark` | ' | Yellow | Electric trail and impact |
| `GroundDust` | , | Brown | Ground trail and impact |
| `FlyingWisp` | ~ | Pale | Flying trail and impact |
| `PsychicSpark` | ° | Magenta | Psychic trail and impact |
| `NormalHit` | * | Gray | Impact for Normal physical attacks (Pinch, Snip, Headbutt, etc.) |

## Effect kinds

| Effect | Format | Meaning |
|---|---|---|
| Damage | `DMG n` | Removes `n` HP from the target. |
| Heal | `HEAL n` | Restores `n` HP to the caster. |
| Buff | `ATK +m% / dt` or `DEF +m% / dt` | Boosts the caster's attack or defense by `m%` for `d` turns. |

## Normal

Physical, mostly cheap. Reliable when mana is low — plus the bulk of the support kit.

| Name | Animation | Effect | Mana |
|---|---|---|---:|
| Pinch ⭐ | Dash | DMG 5 | 0 |
| Scuttle Strike | Dash | DMG 4 | 0 |
| Headbutt | Dash | DMG 6 | 0 |
| Tail Whip | Dash | DMG 5 | 1 |
| Bite | Dash | DMG 7 | 1 |
| Snip ⭐ | Jump | DMG 8 | 2 |
| Shell Bash | Dash | DMG 9 | 3 |
| Claw Crush | Jump | DMG 12 | 4 |
| Double Snip | Jump | DMG 14 | 6 |
| Final Pinch | Jump | DMG 18 | 7 |
| Mend | SelfCast | HEAL 10 | 4 |
| First Aid | SelfCast | HEAL 15 | 6 |
| Greater Mend | SelfCast | HEAL 22 | 10 |
| Sharpen | SelfCast | ATK +25% / 3t | 4 |

## Fire

Aggressive, projectile-heavy. Tends toward higher cost.

| Name | Animation | Effect | Mana |
|---|---|---|---:|
| Ember | Throw(Fire) | DMG 6 | 2 |
| Cinder Spit | Throw(Fire) | DMG 7 | 3 |
| Heatwave | Throw(Fire) | DMG 8 | 4 |
| Flame Dash | Dash | DMG 9 | 4 |
| Fireball | Throw(Fire) | DMG 11 | 5 |
| Sunflare | Throw(Fire) | DMG 13 | 6 |
| Lava Lob | Throw(Fire) | DMG 14 | 6 |
| Pyre Charge | Dash | DMG 15 | 7 |
| Magma Crush | Jump | DMG 17 | 8 |
| Inferno | Throw(Fire) | DMG 21 | 10 |

## Water

Balanced. Strong efficiency at low cost; ramps to a top-tier finisher. Includes a cheap entry-level heal.

| Name | Animation | Effect | Mana |
|---|---|---|---:|
| Splash | Throw(Water) | DMG 4 | 1 |
| Bubble ⭐ | Throw(Water) | DMG 7 | 3 |
| Riptide | Dash | DMG 10 | 5 |
| Whirlpool | Throw(Water) | DMG 12 | 6 |
| Tidal Slam | Jump | DMG 13 | 5 |
| Geyser | Jump | DMG 15 | 7 |
| Tsunami | Throw(Water) | DMG 22 | 12 |
| Salve | SelfCast | HEAL 6 | 2 |

## Ice

Cold attacks split out of Water. Sparse so far but punches above its weight at the top end.

| Name | Animation | Effect | Mana |
|---|---|---|---:|
| Frostbite | Dash | DMG 8 | 4 |
| Ice Shard | Throw(Water) | DMG 9 | 4 |
| Hailstorm | Throw(Water) | DMG 17 | 9 |

## Electric

Lightning attacks. Mid-cost projectiles that scale to a heavy ultimate.

| Name | Animation | Effect | Mana |
|---|---|---|---:|
| Spark | Throw(Electric) | DMG 5 | 2 |
| Static Charge | Dash | DMG 7 | 3 |
| Thunderclap | Throw(Electric) | DMG 9 | 4 |
| Lightning Bolt | Throw(Electric) | DMG 12 | 6 |
| Storm Strike | Throw(Electric) | DMG 16 | 8 |
| Sky Splitter | Throw(Electric) | DMG 22 | 11 |

## Ground

Heavy melee. Few projectiles; biggest single-hit numbers in the game. Also home to the defense buff.

| Name | Animation | Effect | Mana |
|---|---|---|---:|
| Granite Shell | Dash | DMG 6 | 1 |
| Sandstorm | Dash | DMG 7 | 3 |
| Quake Step | Dash | DMG 9 | 4 |
| Stone Slam | Jump | DMG 10 | 4 |
| Iron Pinch | Dash | DMG 11 | 4 |
| Crystal Spike | Jump | DMG 13 | 6 |
| Rockfall | Jump | DMG 14 | 6 |
| Boulder Press | Jump | DMG 16 | 8 |
| Earthquake | Jump | DMG 18 | 9 |
| Tectonic Crush | Jump | DMG 24 | 12 |
| Carapace | SelfCast | DEF +30% / 3t | 4 |

## Flying

Wind-based melee.

| Name | Animation | Effect | Mana |
|---|---|---|---:|
| Gust | Jump | DMG 6 | 2 |
| Tornado | Jump | DMG 14 | 7 |

## Psychic

Cosmic energy attacks. Heaviest mana costs; biggest projectiles.

| Name | Animation | Effect | Mana |
|---|---|---|---:|
| Cosmic Orb ⭐ | Throw(EnergyBall) | DMG 14 | 8 |
| Star Lance | Throw(EnergyBall) | DMG 19 | 10 |

⭐ = starter attack, equipped from the start of a run.

## Balance notes

### Damage
- **Free attacks (cost 0)** sit in the 4–6 damage range — meant as a fallback when out of mana.
- **Cheap (1–3 mana)** ranges 5–9 damage; the most efficient damage-per-mana lives here.
- **Mid (4–7 mana)** ranges 8–15 damage; the workhorse band.
- **Heavy (8–10 mana)** ranges 14–21 damage; commits a turn to a big hit.
- **Ultimate (11–12 mana)** ranges 22–24 damage; one-per-fight finishers.

### Support
- **Heals** scale at roughly *2 HP per mana* (Salve 6/2, Mend 10/4, First Aid 15/6, Greater Mend 22/10) — slightly worse than damage-per-mana on offensive attacks, since the trade is "save your run" not "kill faster."
- **Buffs** cost a flat 4 mana for a 3-turn window. Profitable when you expect to take or land at least three turns; a wasted activation is a real cost.
- Within each band, *efficient* picks (high damage-for-cost) are typically `Dash` or short `Throw`s; *flashy* picks (lower efficiency) trade numbers for animation impact and will pick up status effects in future updates.
