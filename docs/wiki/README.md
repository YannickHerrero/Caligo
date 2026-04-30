# Caligo Wiki

Reference for the game's content and systems. Pages are kept in sync with the data defined under `src/data/` and `src/fight/`.

## Contents

- [Attacks](attacks.md) — every attack the player (or enemies) can use, with damage, mana cost, element, and animation.
- [Items](items.md) — consumables, attack stones, and utility items.
- [Trinkets](trinkets.md) — equippable trinkets and their bonuses.
- [Enemies](enemies.md) — bestiary of foes encountered on the map.
- [Starters](starters.md) — the three crabs you can begin a run with.
- [Settings](settings.md) — theme and other user preferences.

## Conventions

- **Damage** is the raw HP removed from the target on hit. Elemental resistances and modifiers (when implemented) apply on top.
- **Mana** is the MP cost paid up front. Free attacks (cost 0) exist to give players an option even when out of mana.
- **Element** colors the attack and (eventually) interacts with enemy weaknesses.
- **Animation** is the visual: `Dash` (slide in and back), `Jump` (arc onto the target), or `Throw(kind)` (projectile).
