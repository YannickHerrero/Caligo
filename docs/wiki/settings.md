# Settings

User preferences that affect rendering and input. Stored in a process-wide `RwLock<Settings>` (no on-disk persistence yet).

Source of truth: [`src/settings.rs`](../../src/settings.rs).

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
| `Element::Air` (labels) | pale cyan-blue | steel blue |
| `ProjectileKind::Electric` (lightning bolt) | bright yellow | deep amber |
| `ParticleKind::AirWisp` (Air trails / impact) | pale cyan | steel blue |
| `ParticleKind::NeutralHit` (Neutral impact mark) | light gray | mid gray |

Other colors (Fire orange, Water blue, Earth brown, EnergyBall purple, Hearts pink, Triangles red, Circles blue, etc.) are saturated enough to read on both backgrounds and are kept identical across themes.

Future settings (audio, key bindings) are expected to live alongside `Theme` in the same `Settings` struct and the same screen.
