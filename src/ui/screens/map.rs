use crate::data::{enemies, starters};
use crate::map::{self, MapGraph, NodeId, NodeKind};
use crate::player::Player;
use crate::run::Run;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::{
    FightScreen, PlaceholderNodeScreen, PlayerInfoScreen, SelectScreen, StartScreen,
    TransitionKind, TransitionScreen,
};
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

const SCROLL_STEP: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMenuState {
    Browsing,
    Confirming,
    Abandoning,
}

/// Where the MapScreen returns to when the player abandons. Real runs
/// hand control back to the start menu; debug-mode visits return to the
/// SelectScreen test harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapOrigin {
    Run,
    DebugSelect,
}

#[derive(Clone)]
pub struct MapScreen {
    pub run: Run,
    pub origin: MapOrigin,
    pub cursor: Option<NodeId>,
    pub tick: u32,
    pub menu_state: MapMenuState,
    scroll: i32,
    last_viewport_height: u16,
}

impl MapScreen {
    pub fn new() -> Self {
        // Default constructor used by --debug flows that bypass StarterSelect.
        // Falls back to the first starter so the screen is functional in
        // isolation.
        let starter = starters::all_starters().remove(0);
        Self::with_run_and_origin(Run::new(starter, map::generate()), MapOrigin::DebugSelect)
    }

    pub fn with_run(run: Run) -> Self {
        Self::with_run_and_origin(run, MapOrigin::Run)
    }

    fn with_run_and_origin(run: Run, origin: MapOrigin) -> Self {
        let cursor = pick_default_cursor(&run.map);
        Self {
            run,
            origin,
            cursor,
            tick: 0,
            menu_state: MapMenuState::Browsing,
            scroll: 0,
            last_viewport_height: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        if matches!(key, KeyCode::Tab) && self.menu_state == MapMenuState::Browsing {
            let from = std::mem::replace(self, MapScreen::new());
            return Transition::Goto(Screen::PlayerInfo(PlayerInfoScreen::new(from)));
        }
        match self.menu_state {
            MapMenuState::Browsing => self.handle_browsing(key),
            MapMenuState::Confirming => self.handle_confirming(key, player),
            MapMenuState::Abandoning => self.handle_abandoning(key),
        }
    }

    fn handle_browsing(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.menu_state = MapMenuState::Abandoning;
                return Transition::Stay;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.move_cursor(-1);
                self.center_scroll_on_cursor();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.move_cursor(1);
                self.center_scroll_on_cursor();
            }
            KeyCode::Up | KeyCode::Char('k') => self.scroll_by(-SCROLL_STEP),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_by(SCROLL_STEP),
            KeyCode::PageUp => self.scroll_by(-(self.last_viewport_height as i32)),
            KeyCode::PageDown => self.scroll_by(self.last_viewport_height as i32),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = self.max_scroll(),
            KeyCode::Enter => {
                if self.cursor.is_some() {
                    self.menu_state = MapMenuState::Confirming;
                    self.center_scroll_on_cursor();
                }
            }
            _ => {}
        }
        Transition::Stay
    }

    fn handle_confirming(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        match key {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                self.menu_state = MapMenuState::Browsing;
                Transition::Stay
            }
            KeyCode::Enter => {
                let Some(id) = self.cursor else {
                    self.menu_state = MapMenuState::Browsing;
                    return Transition::Stay;
                };
                let kind = self.run.map.node(id).kind;
                if !self.run.map.select(id) {
                    self.menu_state = MapMenuState::Browsing;
                    return Transition::Stay;
                }
                self.menu_state = MapMenuState::Browsing;

                // Move the current map forward so the run persists; clone
                // for the transition's fade visual.
                let map_owned = std::mem::replace(self, MapScreen::new());
                let map_for_fade = map_owned.clone();
                let to = build_node_screen(player, Box::new(map_owned), kind);
                let transition = TransitionScreen::new(
                    Screen::Map(map_for_fade),
                    to,
                    TransitionKind::from(kind),
                );
                Transition::Goto(Screen::Transition(transition))
            }
            _ => Transition::Stay,
        }
    }

    fn handle_abandoning(&mut self, key: KeyCode) -> Transition {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => match self.origin {
                MapOrigin::Run => Transition::Goto(Screen::Start(StartScreen::new())),
                MapOrigin::DebugSelect => Transition::Goto(Screen::Select(SelectScreen::new())),
            },
            KeyCode::Char('n')
            | KeyCode::Char('N')
            | KeyCode::Esc
            | KeyCode::Backspace
            | KeyCode::Char('q') => {
                self.menu_state = MapMenuState::Browsing;
                Transition::Stay
            }
            _ => Transition::Stay,
        }
    }

    fn scroll_by(&mut self, delta: i32) {
        let max = self.max_scroll();
        self.scroll = (self.scroll + delta).clamp(0, max);
    }

    fn max_scroll(&self) -> i32 {
        let total = widgets::map_virtual_height();
        (total - self.last_viewport_height as i32).max(0)
    }

    fn center_scroll_on_cursor(&mut self) {
        if self.last_viewport_height == 0 {
            return;
        }
        self.scroll =
            widgets::compute_map_scroll(&self.run.map, self.cursor, self.last_viewport_height);
    }

    fn move_cursor(&mut self, delta: i32) {
        let reachable = sorted_reachable(&self.run.map);
        if reachable.is_empty() {
            self.cursor = None;
            return;
        }
        let len = reachable.len() as i32;
        let pos = self
            .cursor
            .and_then(|id| reachable.iter().position(|&n| n == id))
            .unwrap_or(0) as i32;
        let new_pos = (pos + delta).rem_euclid(len) as usize;
        self.cursor = Some(reachable[new_pos]);
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        self.tick = self.tick.wrapping_add(1);
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(6),
            ])
            .split(area);

        let header_area = chunks[0];
        let map_area = chunks[1];
        let info_area = chunks[2];

        let pulse = pulse_phase(self.tick);
        if self.last_viewport_height != map_area.height {
            self.last_viewport_height = map_area.height;
            self.center_scroll_on_cursor();
        }
        // Re-clamp in case viewport shrank below current scroll.
        let max = self.max_scroll();
        if self.scroll > max {
            self.scroll = max;
        }
        let scroll = self.scroll;
        widgets::render_map_header(frame, &self.run.map, header_area);
        widgets::render_map_edges(frame, &self.run.map, scroll, map_area);
        widgets::render_map_nodes(frame, &self.run.map, self.cursor, pulse, scroll, map_area);
        widgets::render_map_info(frame, &self.run.map, self.cursor, info_area);
        if self.menu_state == MapMenuState::Confirming {
            if let Some(id) = self.cursor {
                widgets::render_map_confirm(frame, self.run.map.node(id), area);
            }
        }
        if self.menu_state == MapMenuState::Abandoning {
            render_abandon_popup(frame, self.origin, area);
        }
    }
}

fn render_abandon_popup(frame: &mut Frame, origin: MapOrigin, area: Rect) {
    let popup_w: u16 = 44.min(area.width.saturating_sub(4)).max(20);
    let popup_h: u16 = 7.min(area.height.saturating_sub(2)).max(5);
    if popup_w < 20 || popup_h < 5 {
        return;
    }
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(popup_w)) / 2,
        y: area.y + (area.height.saturating_sub(popup_h)) / 2,
        width: popup_w,
        height: popup_h,
    };

    // Dim everything outside the popup so it feels modal, then clear the
    // popup area so map glyphs don't bleed through.
    let buf = frame.buffer_mut();
    for y in area.y..(area.y + area.height) {
        for x in area.x..(area.x + area.width) {
            if x >= popup.x
                && x < popup.x + popup.width
                && y >= popup.y
                && y < popup.y + popup.height
            {
                continue;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                let fg = match cell.fg {
                    Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
                    other => other,
                };
                cell.set_fg(fg);
            }
        }
    }
    for y in popup.y..(popup.y + popup.height) {
        for x in popup.x..(popup.x + popup.width) {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ').set_style(Style::default());
            }
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(220, 80, 80)))
        .title(Span::styled(
            " Abandon? ",
            Style::default()
                .fg(Color::Rgb(220, 80, 80))
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    if inner.height == 0 {
        return;
    }

    let body_text = match origin {
        MapOrigin::Run => "End the run? Progress will be lost.",
        MapOrigin::DebugSelect => "Leave the map?",
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            body_text,
            Style::default().fg(Color::Gray),
        )))
        .alignment(ratatui::layout::Alignment::Center),
        chunks[0],
    );

    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let prompt = Line::from(vec![
        Span::styled("Y", key),
        Span::styled(" / ", dim),
        Span::styled("Enter", key),
        Span::styled("  abandon   ", dim),
        Span::styled("N", key),
        Span::styled(" / ", dim),
        Span::styled("Esc", key),
        Span::styled("  cancel", dim),
    ]);
    frame.render_widget(
        Paragraph::new(prompt).alignment(ratatui::layout::Alignment::Center),
        chunks[2],
    );
}

fn build_node_screen(player: &mut Player, map: Box<MapScreen>, kind: NodeKind) -> Screen {
    let mut rng = rand::thread_rng();
    match kind {
        NodeKind::EasyFight | NodeKind::NormalFight | NodeKind::EliteFight | NodeKind::Boss => {
            let enemy = enemies::pick_for_node(kind, &mut rng)
                .unwrap_or_else(crate::data::enemies::slime);
            Screen::Fight(FightScreen::from_map(player, map, enemy, kind))
        }
        NodeKind::Camp | NodeKind::Shop | NodeKind::Mystery => {
            Screen::PlaceholderNode(PlaceholderNodeScreen::new(map, kind, player))
        }
    }
}

fn pulse_phase(tick: u32) -> f32 {
    let t = (tick % 24) as f32 / 24.0;
    let s = (t * std::f32::consts::TAU).sin();
    0.5 + 0.5 * s
}

fn sorted_reachable(graph: &MapGraph) -> Vec<NodeId> {
    let mut ids = graph.reachable();
    ids.sort_by_key(|&id| graph.node(id).column);
    ids
}

fn pick_default_cursor(graph: &MapGraph) -> Option<NodeId> {
    let reachable = sorted_reachable(graph);
    if reachable.is_empty() {
        return None;
    }
    Some(reachable[reachable.len() / 2])
}
