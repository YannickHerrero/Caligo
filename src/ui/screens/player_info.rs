use crate::crab::Crab;
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::MapScreen;
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders},
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
}

impl PlayerInfoScreen {
    pub fn new(map: MapScreen) -> Self {
        Self {
            focus: InfoFocus::Attacks,
            map: Some(Box::new(map)),
            crab: Crab::new((0.0, 0.0), 95),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        match key {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Tab => self.return_to_map(),
            _ => Transition::Stay,
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

        widgets::render_crab_panel(frame, &self.crab, left[0]);
        widgets::render_stats_panel(frame, player, left[1]);
        draw_panel(
            frame,
            " Attacks ",
            right[0],
            self.focus == InfoFocus::Attacks,
        );
        draw_panel(
            frame,
            " Inventory ",
            right[1],
            self.focus == InfoFocus::Inventory,
        );
        draw_panel(frame, " Info ", info_strip, false);
    }
}

fn draw_panel(frame: &mut Frame, title: &str, area: Rect, focused: bool) {
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            title.to_string(),
            Style::default().fg(if focused { Color::Yellow } else { Color::Gray }),
        ));
    frame.render_widget(block, area);
}
