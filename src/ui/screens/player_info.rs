use crate::crab::Crab;
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::MapScreen;
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoFocus {
    Attacks,
    Inventory,
}

pub struct PlayerInfoScreen {
    pub focus: InfoFocus,
    pub map: Option<Box<MapScreen>>,
    crab: Crab,
    pub attack_cursor: usize,
    pub attack_scroll: usize,
    pub item_cursor: usize,
    pub item_scroll: usize,
    last_attacks_height: u16,
    last_inventory_height: u16,
}

impl PlayerInfoScreen {
    pub fn new(map: MapScreen) -> Self {
        Self {
            focus: InfoFocus::Attacks,
            map: Some(Box::new(map)),
            crab: Crab::new((0.0, 0.0), 95),
            attack_cursor: 0,
            attack_scroll: 0,
            item_cursor: 0,
            item_scroll: 0,
            last_attacks_height: 0,
            last_inventory_height: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        match key {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Tab => return self.return_to_map(),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_focused(-1, player),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_focused(1, player),
            KeyCode::Left | KeyCode::Char('h') => self.focus = InfoFocus::Attacks,
            KeyCode::Right | KeyCode::Char('l') => self.focus = InfoFocus::Inventory,
            _ => {}
        }
        Transition::Stay
    }

    fn scroll_focused(&mut self, delta: i32, player: &Player) {
        match self.focus {
            InfoFocus::Attacks => scroll_list(
                &mut self.attack_cursor,
                &mut self.attack_scroll,
                player.owned_attacks.len(),
                self.last_attacks_height as usize,
                delta,
            ),
            InfoFocus::Inventory => scroll_list(
                &mut self.item_cursor,
                &mut self.item_scroll,
                player.inventory.len(),
                self.last_inventory_height as usize,
                delta,
            ),
        }
    }

    fn return_to_map(&mut self) -> Transition {
        match self.map.take() {
            Some(map) => Transition::Goto(Screen::Map(*map)),
            None => Transition::Goto(Screen::Map(MapScreen::new())),
        }
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, player: &Player) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(8), Constraint::Length(3)])
            .split(area);
        let body_area = chunks[0];
        let info_strip = chunks[1];

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(body_area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(columns[0]);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(columns[1]);

        let attacks_area = right[0];
        let inventory_area = right[1];
        self.last_attacks_height = attacks_area.height.saturating_sub(2);
        self.last_inventory_height = inventory_area.height.saturating_sub(2);

        widgets::render_crab_panel(frame, &self.crab, left[0]);
        widgets::render_stats_panel(frame, player, left[1]);
        widgets::render_attacks_panel(
            frame,
            player,
            self.attack_cursor,
            self.attack_scroll,
            self.focus == InfoFocus::Attacks,
            attacks_area,
        );
        widgets::render_inventory_panel(
            frame,
            player,
            self.item_cursor,
            self.item_scroll,
            self.focus == InfoFocus::Inventory,
            inventory_area,
        );

        match self.focus {
            InfoFocus::Attacks => {
                let popup_attack = player.owned_attacks.get(self.attack_cursor);
                widgets::render_info_strip(frame, popup_attack, info_strip);
            }
            InfoFocus::Inventory => match player.inventory.get(self.item_cursor) {
                Some(stack) => widgets::render_item_info_strip(frame, &stack.item, info_strip),
                None => widgets::render_info_strip(frame, None, info_strip),
            },
        }
    }
}

fn scroll_list(cursor: &mut usize, scroll: &mut usize, len: usize, visible: usize, delta: i32) {
    if len == 0 {
        *cursor = 0;
        *scroll = 0;
        return;
    }
    let new_cursor = (*cursor as i32 + delta).clamp(0, len as i32 - 1) as usize;
    *cursor = new_cursor;
    if visible > 0 {
        if *cursor < *scroll {
            *scroll = *cursor;
        } else if *cursor >= *scroll + visible {
            *scroll = *cursor + 1 - visible;
        }
    }
}
