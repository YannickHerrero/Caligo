# Caligo

> *cāligō*, Latin: gloom, fog, dark mist; mental obscurity.

A roguelike crab dungeon crawler that runs in your terminal.

> **Status:** Early development. The fight-scene MVP is in place; the map screen and core combat systems are up next.

## About

Caligo is a turn-based roguelike where you guide a crab through a series of dungeons. Each run takes you across a branching map of rooms — fights, shops, events — building toward a boss at the floor's end. Combat plays out in a JRPG-style fight scene: you pick from a menu of attacks, items, or flee, while animations play in the scene above.

The art style is intentionally minimalist — ASCII art rendered through [ratatui](https://github.com/ratatui/ratatui), with a focus on snappy animations and terminal-friendly visuals.

## Inspiration

- **[kanitomo](https://github.com/YannickHerrero/kanitomo)** — a terminal pet crab. The crab character, environment (day/night, clouds, ground themes), and overall layout are ported from there.
- **Slay the Spire** — for the branching map of rooms.
- **Pokémon** — for the main → submenu → action flow.
- **Final Fantasy** — for the fixed-action fight panel feel.

## Features

### Implemented

- Animated 4-line ASCII crab with five mood expressions and physics-driven idle motion
- Day/night cycle with sun, moon, drifting clouds, and stars at night
- Four ground themes: Beach, Garden, Rocky, Minimal
- Fight scene with top bar, scene area, and bottom action panel
- HP bars for player and enemy
- Three-tier action menu: **Attack / Item / Flee**
  - Attack submenu: 2×2 grid, up to 4 attacks
  - Item submenu: scrollable list
- Three attack animations:
  - **Jump** — crab arcs onto the enemy
  - **Dash** — crab slides to the enemy and back
  - **Throw** — crab launches a projectile in an arc
- Variable projectile types, each with their own sprite, color, and size:
  - Water (1×1, blue droplet)
  - Fire (2×2, orange flame)
  - Electric (1×3, yellow bolt)
  - Energy Ball (3×3, purple orb)

### Roadmap

- Slay-the-Spire-style branching map with room types (combat, shop, event, rest, boss)
- Real attack and item systems (damage, costs, status effects)
- Turn order and enemy AI
- Multiple enemy types with their own sprites and movesets
- Dungeon generation and persistent run state

## Getting started

You'll need a Rust toolchain (1.74 or later recommended).

```bash
git clone <this-repo>
cd caligo
cargo run
```

A reasonably wide terminal (≥80 columns) is recommended.

## Controls

**Main menu**

| Key | Action |
|---|---|
| `↑` / `k` | Cycle up |
| `↓` / `j` | Cycle down |
| `Enter` | Confirm |
| `q` / `Esc` | Quit |

**Attack submenu (2×2 grid)**

| Key | Action |
|---|---|
| `↑↓←→` / `hjkl` | Move cursor |
| `Enter` | Use attack |
| `Esc` / `Backspace` / `q` | Back to main menu |

**Item submenu (scrollable list)**

| Key | Action |
|---|---|
| `↑` / `k` | Up |
| `↓` / `j` | Down |
| `Esc` / `Backspace` / `q` | Back to main menu |

While an attack animation plays, input is locked until it finishes.

## Project structure

```
src/
├── main.rs
├── crab/           # crab entity, mood, animation
├── environment/    # day/night cycle, sky, ground themes
├── fight/          # fight state, actions, attacks, items, projectiles, animations
└── ui/
    ├── app.rs      # main loop, event handling, draw orchestration
    └── widgets/    # scene rendering (crab, enemy, ground), fight panels, helpers
```

## Contributing

This is a hobby project in active development. If something catches your eye, feel free to open an issue or PR.

## License

TBD.
