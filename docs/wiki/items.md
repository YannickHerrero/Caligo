# Items

Anything that lives in the inventory and is selected from the **Item** menu in a fight (or used outside of combat).

Source of truth: [`src/fight/item.rs`](../../src/fight/item.rs).

## Potions

Restore HP or MP up to the current cap. Consumed on use.

| Item | Effect |
|---|---|
| Small HP Potion | Restores 10 HP. |
| Large HP Potion | Restores 30 HP. |
| Small Mana Potion | Restores 6 MP. |
| Large Mana Potion | Restores 15 MP. |

## Attack Stones

Single-use stones that teach the named attack. Using a stone permanently adds the attack to the player's owned pool. If the attack is already known, the stone reports `AlreadyKnown` and is not consumed (see `Player::use_inventory_item`).

| Item | Effect |
|---|---|
| Stone of *<attack>* | Teaches the named attack from the [attack library](attacks.md). |

## Utility

Special-purpose items. Some only function in combat.

| Item | Effect |
|---|---|
| Revive Pearl | Auto-revives the crab when defeated in combat. Combat-only. |
| Escape Token | Guarantees escape from a fight. Combat-only. |
| Gold Pouch | Opens to grant 25 gold. Usable anywhere. |

## Trinkets

Trinkets are equippable items that grant passive bonuses while equipped — see the dedicated [Trinkets page](trinkets.md).
