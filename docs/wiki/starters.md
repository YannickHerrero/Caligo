# Starters

The three creatures the player can begin a run with. Different species, not recolors of the same crab — Pinchy is the original tidepool crab, Cinder is a cute flame, and Sprout is a piranha plant on a stem.

Source of truth: [`src/data/starters.rs`](../../src/data/starters.rs).

> **In-game preview:** open *Catalogue* and `Tab` over to the *Starters* page to flip through them with their sprite, type, and starting moves.

## The three starters

| Starter | Creature | Type | Starting moves | Flavor |
|---|---|---|---|---|
| Pinchy | Crab | Water | Pinch, Bubble, Snip, Cosmic Orb | The default tidepool crab. Balanced and sturdy, generalist kit. |
| Cinder | Cute flame | Fire | Pinch, Ember, Snip, Cinder Spit | A spry flame with a face. Aggressive opener, fragile shell. |
| Sprout | Piranha plant | Grass | Pinch, Vine Whip, Snip, Leaf Slash | A piranha plant on a stout stem. Patient — bites when you're not looking. |

The classic Fire / Water / Grass triangle — Cinder beats Sprout, Sprout beats Pinchy, Pinchy beats Cinder.

## What a Starter carries

```rust
struct Starter {
    name, description,
    primary_type,
    starting_attacks: Vec<&'static str>, // attack names from the library
    sprite: Vec<String>,                 // unique ASCII art per starter
    palette: ThemedColor,
}
```

Stats and inventory aren't on the starter yet — `Player::new()` still constructs the in-run stats independent of the starter pick. Wiring the starter selection into the actual run is a follow-up.

## Roadmap

- Starter select screen on game start.
- Per-starter base stats (HP / mana) and starting trinkets.
- Signature mid-game attacks per starter (e.g. Cinder learns Pyre Charge as a milestone).
