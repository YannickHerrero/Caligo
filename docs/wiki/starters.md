# Starters

The three crabs the player can begin a run with. Each is the same crab body in a different color with a different elemental type and a different starting moveset.

Source of truth: [`src/data/starters.rs`](../../src/data/starters.rs).

> **In-game preview:** open *Catalogue* and `Tab` over to the *Starters* page to flip through them with their sprite, type, and starting moves.

## The three starters

| Starter | Type | Starting moves | Flavor |
|---|---|---|---|
| Pinchy | Water | Pinch, Bubble, Snip, Cosmic Orb | The default tidepool crab. Balanced and sturdy, generalist kit. |
| Cinder | Fire | Pinch, Ember, Snip, Cinder Spit | Hatched in a tidepool that ran a little hot. Aggressive, fragile shell. |
| Sprout | Grass | Pinch, Vine Whip, Snip, Leaf Slash | A crab who has clearly been spending time in the kelp. Slow but steady. |

The classic Fire / Water / Grass triangle — Cinder beats Sprout, Sprout beats Pinchy, Pinchy beats Cinder.

## What a Starter carries

```rust
struct Starter {
    name, description,
    primary_type,
    starting_attacks: Vec<&'static str>, // attack names from the library
    sprite, palette,                     // crab body + ThemedColor
}
```

Stats and inventory aren't on the starter yet — `Player::new()` still constructs the in-run stats independent of the starter pick. Wiring the starter selection into the actual run is a follow-up.

## Roadmap

- Starter select screen on game start.
- Per-starter base stats (HP / mana) and starting trinkets.
- Signature mid-game attacks per starter (e.g. Cinder learns Pyre Charge as a milestone).
