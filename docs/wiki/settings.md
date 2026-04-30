# Settings

User preferences that affect rendering and input. Held in a process-wide `RwLock<Settings>` and persisted to disk so choices survive between runs.

Source of truth: [`src/settings.rs`](../../src/settings.rs).

## Storage

Settings are read on startup and written every time a setting changes. The file lives at:

- `$XDG_CONFIG_HOME/caligo/settings` if that env var is set.
- Otherwise `$HOME/.config/caligo/settings`.
- If neither is set the file is silently skipped — no crash, just no persistence.

Format is one `key=value` line per setting (no TOML/JSON dependency for now):

```
theme=light
```

Order of precedence at startup:

1. Default (`Theme::Dark`).
2. Whatever the config file contains.
3. `CALIGO_THEME=light|dark` env var, applied last and **not written back** — useful for one-off launches without changing the saved choice.

## Theme

| Value | When to pick |
|---|---|
| `Dark` *(default)* | Dark terminal background. The full-saturation palette is built around this. |
| `Light` | Light terminal background. Pale colors that disappear on a white background are swapped for darker variants. |

### How to set it

- **At startup**: set the `CALIGO_THEME` env var to `light` or `dark` before launching:
  ```bash
  CALIGO_THEME=light cargo run
  ```
- **In-game**: open *Settings* from the home menu and toggle with `← → / Enter`.

### Which colors actually adapt

The audit so far covers the colors that go invisible on a light background:

| Where | Dark | Light |
|---|---|---|
| `Element::Flying` (labels) | pale cyan-blue | steel blue |
| `Element::Ice` (labels) | cyan | deep teal |
| `Element::Electric` (labels) | bright yellow | deep amber |
| `ProjectileKind::Electric` (lightning bolt) | bright yellow | deep amber |
| `ParticleKind::FlyingWisp` (Flying trails / impact) | pale cyan | steel blue |
| `ParticleKind::IceShard` (Ice trails / impact) | cyan | deep teal |
| `ParticleKind::ElectricSpark` (Electric trails / impact) | bright yellow | deep amber |
| `ParticleKind::NormalHit` (Normal impact mark) | light gray | mid gray |

Other colors (Fire orange, Water blue, Earth brown, EnergyBall purple, Hearts pink, Triangles red, Circles blue, etc.) are saturated enough to read on both backgrounds and are kept identical across themes.

Future settings (audio, key bindings) are expected to live alongside `Theme` in the same `Settings` struct and the same screen.
