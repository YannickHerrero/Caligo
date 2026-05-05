use crate::fight::{Item, ItemStack, PotionSize, TrinketKind};
use crate::map::NodeKind;
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::MapScreen;
use crossterm::event::KeyCode;
use rand::Rng;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone)]
struct ShopOffer {
    item: Item,
    price: u32,
    sold: bool,
}

/// In-run shop node. Shows a five-slot stock rolled at entry, lets the
/// player buy with run gold (`Player.gold`), and continues to the next
/// map node when dismissed.
pub struct InRunShopScreen {
    pub map: Option<Box<MapScreen>>,
    stock: Vec<ShopOffer>,
    selected: usize,
    message: Option<String>,
}

impl InRunShopScreen {
    pub fn new(map: Box<MapScreen>) -> Self {
        let floor = current_floor(&map);
        let mut rng = rand::thread_rng();
        let stock = roll_stock(floor, &mut rng);
        Self {
            map: Some(map),
            stock,
            selected: 0,
            message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        let len = self.stock.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.leave(),
            KeyCode::Up | KeyCode::Char('k') => {
                if len > 0 {
                    self.selected = (self.selected + len - 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len > 0 {
                    self.selected = (self.selected + 1) % len;
                }
                self.message = None;
                Transition::Stay
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.try_buy(player);
                Transition::Stay
            }
            // 'c' as a Continue shortcut, separate from Esc so the player
            // doesn't worry Esc means "abandon".
            KeyCode::Char('c') => self.leave(),
            _ => Transition::Stay,
        }
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, player: &Player) {
        let area = frame.area();
        if area.width == 0 || area.height == 0 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title
                Constraint::Length(1), // subtitle / gold
                Constraint::Length(1), // spacer
                Constraint::Min(8),    // shop list
                Constraint::Length(1), // message strip
                Constraint::Length(1), // hint
            ])
            .split(area);

        render_title(frame, chunks[0]);
        render_gold(frame, player.gold, chunks[1]);
        render_stock(frame, &self.stock, self.selected, player.gold, chunks[3]);
        if let Some(msg) = self.message.as_deref() {
            render_message(frame, msg, chunks[4]);
        }
        render_hint(frame, chunks[5]);
    }

    fn leave(&mut self) -> Transition {
        match self.map.take() {
            Some(map) => Transition::Goto(Screen::Map(*map)),
            None => Transition::Stay,
        }
    }

    fn try_buy(&mut self, player: &mut Player) {
        let Some(offer) = self.stock.get_mut(self.selected) else {
            return;
        };
        if offer.sold {
            self.message = Some("Already sold.".to_string());
            return;
        }
        if player.gold < offer.price {
            self.message = Some(format!(
                "Need {} gold ({} short).",
                offer.price,
                offer.price - player.gold
            ));
            return;
        }
        player.gold -= offer.price;
        let name = offer.item.name();
        // Stack-add into the player's inventory.
        if let Some(stack) = player
            .inventory
            .iter_mut()
            .find(|s| same_item(&s.item, &offer.item))
        {
            stack.count = stack.count.saturating_add(1);
        } else {
            player
                .inventory
                .push(ItemStack::new(offer.item.clone(), 1));
        }
        offer.sold = true;
        self.message = Some(format!("Bought {} for {} gold.", name, offer.price));
    }
}

fn current_floor(map: &MapScreen) -> u8 {
    map.run
        .map
        .current
        .map(|id| map.run.map.node(id).floor)
        .unwrap_or(0)
}

fn roll_stock<R: Rng>(floor: u8, rng: &mut R) -> Vec<ShopOffer> {
    let mut out = Vec::with_capacity(5);

    // HP Potion: 60% Small, 40% Large.
    out.push(if rng.gen_bool(0.6) {
        ShopOffer {
            item: Item::HpPotion(PotionSize::Small),
            price: 15,
            sold: false,
        }
    } else {
        ShopOffer {
            item: Item::HpPotion(PotionSize::Large),
            price: 40,
            sold: false,
        }
    });

    // Mana Potion: 60% Small, 40% Large.
    out.push(if rng.gen_bool(0.6) {
        ShopOffer {
            item: Item::ManaPotion(PotionSize::Small),
            price: 12,
            sold: false,
        }
    } else {
        ShopOffer {
            item: Item::ManaPotion(PotionSize::Large),
            price: 30,
            sold: false,
        }
    });

    // Monster Net.
    out.push(ShopOffer {
        item: Item::MonsterNet,
        price: 25,
        sold: false,
    });

    // Attack Stone, tier weighted by floor depth (deeper => more
    // powerful pool). Reuses the reward-table tier weights.
    let stage = stage_for_floor(floor);
    if let Some(stone) = crate::ui::screens::reward::stone_for_stage(stage, rng) {
        let price = stone_price(&stone);
        out.push(ShopOffer {
            item: stone,
            price,
            sold: false,
        });
    }

    // Trinket — rare slot. One random kind at a flat 60g.
    let kind = match rng.gen_range(0..3) {
        0 => TrinketKind::HeartCharm,
        1 => TrinketKind::ManaPearl,
        _ => TrinketKind::LuckyShell,
    };
    out.push(ShopOffer {
        item: Item::Trinket(kind),
        price: 60,
        sold: false,
    });

    out
}

/// Map raw floor depth onto the NodeKind we'd use for tier-weighting
/// stone rolls. Mirrors the tier curve from the reward pool.
fn stage_for_floor(floor: u8) -> NodeKind {
    if floor < 4 {
        NodeKind::EasyFight
    } else if floor < 8 {
        NodeKind::NormalFight
    } else if floor < 12 {
        NodeKind::EliteFight
    } else {
        NodeKind::Boss
    }
}

fn stone_price(item: &Item) -> u32 {
    let Item::AttackStone { attack_name } = item else {
        return 50;
    };
    let Some(attack) = crate::data::attacks::find_by_name(attack_name) else {
        return 50;
    };
    use crate::fight::Effect;
    match attack.effect {
        Effect::Damage(n) => match n {
            0..=7 => 25,
            8..=12 => 50,
            13..=18 => 100,
            _ => 200,
        },
        Effect::Heal(_) => 60,
        Effect::Buff { .. } => 60,
    }
}

fn same_item(a: &Item, b: &Item) -> bool {
    match (a, b) {
        (Item::HpPotion(s1), Item::HpPotion(s2)) => s1 == s2,
        (Item::ManaPotion(s1), Item::ManaPotion(s2)) => s1 == s2,
        (Item::AttackStone { attack_name: n1 }, Item::AttackStone { attack_name: n2 }) => n1 == n2,
        (Item::Trinket(k1), Item::Trinket(k2)) => k1 == k2,
        (Item::Utility(k1), Item::Utility(k2)) => k1 == k2,
        (Item::MonsterNet, Item::MonsterNet) => true,
        _ => false,
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        "Shop",
        Style::default()
            .fg(Color::Rgb(110, 210, 230))
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_gold(frame: &mut Frame, gold: u32, area: Rect) {
    let line = Line::from(vec![
        Span::styled("Gold ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", gold),
            Style::default()
                .fg(Color::Rgb(240, 210, 110))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_stock(
    frame: &mut Frame,
    stock: &[ShopOffer],
    selected: usize,
    gold: u32,
    area: Rect,
) {
    let panel_w = 64.min(area.width.saturating_sub(2)).max(40);
    let needed_h = (stock.len().max(1) as u16) * 2 + 2;
    let panel_h = needed_h.min(area.height);
    let panel = Rect {
        x: area.x + (area.width.saturating_sub(panel_w)) / 2,
        y: area.y + (area.height.saturating_sub(panel_h)) / 2,
        width: panel_w,
        height: panel_h,
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(110, 210, 230)))
        .title(Span::styled(
            " Stock ",
            Style::default()
                .fg(Color::Rgb(110, 210, 230))
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(panel);
    frame.render_widget(block, panel);
    if inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();
    for (idx, offer) in stock.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "\u{25B6} " } else { "  " };
        let name_style = if offer.sold {
            Style::default().fg(Color::DarkGray)
        } else if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let price_style = if offer.sold {
            Style::default().fg(Color::DarkGray)
        } else if gold >= offer.price {
            Style::default()
                .fg(Color::Rgb(240, 210, 110))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let suffix = if offer.sold { "  (sold)" } else { "" };
        lines.push(Line::from(vec![
            Span::styled(cursor, name_style),
            Span::styled(format!("{:<28}", offer.item.name()), name_style),
            Span::styled(format!("{} gold", offer.price), price_style),
            Span::styled(suffix.to_string(), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_message(frame: &mut Frame, message: &str, area: Rect) {
    let line = Line::from(Span::styled(
        message.to_string(),
        Style::default().fg(Color::Rgb(255, 210, 110)),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_hint(frame: &mut Frame, area: Rect) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let hint = Line::from(vec![
        Span::styled("\u{2191}\u{2193}", key),
        Span::styled(" navigate   ", dim),
        Span::styled("Enter", key),
        Span::styled(" buy   ", dim),
        Span::styled("c", key),
        Span::styled(" / ", dim),
        Span::styled("Esc", key),
        Span::styled(" leave", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}

