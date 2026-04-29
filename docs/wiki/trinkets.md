# Trinkets

Trinkets are passive equippables. The player has **2 trinket slots** (`MAX_TRINKETS` in `player::state`). Equipping a trinket toggles it on; selecting it again unequips it.

Source of truth: [`src/fight/item.rs`](../../src/fight/item.rs) and [`src/player/state.rs`](../../src/player/state.rs).

## Available trinkets

| Trinket | Effect |
|---|---|
| Heart Charm | +10 max HP while equipped. |
| Mana Pearl | +5 max MP while equipped. |
| Lucky Shell | Slight luck bonus while equipped. |

## Equip rules

- Up to `MAX_TRINKETS = 2` trinkets can be equipped at once.
- Toggling a trinket through the Item menu equips it into the first free slot, or unequips it if already worn.
- If both slots are full when a new trinket is selected, the trinket in slot 0 is overwritten.
- Unequipping a stat-boosting trinket immediately clamps current HP/MP to the new (lower) cap.

## Stat bonuses

Effective max stats are computed as `base + sum(equipped trinket bonuses)`:

```
max_hp   = base_max_hp   + Σ trinket.bonus_max_hp
max_mana = base_max_mana + Σ trinket.bonus_max_mana
```

The Stats panel in-game shows the trinket contribution next to the relevant slot.
