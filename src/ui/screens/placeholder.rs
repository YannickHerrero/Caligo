use crate::map::NodeKind;
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::MapScreen;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Generic Continue-button screen used for node kinds whose real
/// implementation is deferred. Camp restores the player to full HP/MP
/// when entered; Shop and Mystery just present a "coming soon" panel.
pub struct PlaceholderNodeScreen {
    pub map: Option<Box<MapScreen>>,
    pub kind: NodeKind,
    pub healed_to: Option<u32>,
}

impl PlaceholderNodeScreen {
    pub fn new(map: Box<MapScreen>, kind: NodeKind, player: &mut Player) -> Self {
        let healed_to = if matches!(kind, NodeKind::Camp) {
            player.hp = player.max_hp();
            player.mana = player.max_mana();
            Some(player.hp)
        } else {
            None
        };
        Self {
            map: Some(map),
            kind,
            healed_to,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        match key {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('q') | KeyCode::Esc => {
                match self.map.take() {
                    Some(map) => Transition::Goto(Screen::Map(*map)),
                    None => Transition::Stay,
                }
            }
            _ => Transition::Stay,
        }
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
        let area = frame.area();
        if area.width == 0 || area.height == 0 {
            return;
        }

        let panel_w = 50.min(area.width.saturating_sub(4)).max(20);
        let panel_h: u16 = 9.min(area.height.saturating_sub(2));
        let panel = Rect {
            x: area.x + (area.width.saturating_sub(panel_w)) / 2,
            y: area.y + (area.height.saturating_sub(panel_h)) / 2,
            width: panel_w,
            height: panel_h,
        };

        let (title, accent) = match self.kind {
            NodeKind::Camp => (" Camp ", Color::Rgb(255, 210, 110)),
            NodeKind::Shop => (" Shop ", Color::Rgb(110, 210, 230)),
            NodeKind::Mystery => (" Mystery ", Color::Rgb(190, 130, 230)),
            _ => (" Node ", Color::Gray),
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent))
            .title(Span::styled(
                title,
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(panel);
        frame.render_widget(block, panel);
        if inner.height == 0 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // headline
                Constraint::Min(1),    // body
                Constraint::Length(1), // hint
            ])
            .split(inner);

        let headline_text = match self.kind {
            NodeKind::Camp => "You make camp and rest a while.",
            NodeKind::Shop => "A wandering merchant — closed for now.",
            NodeKind::Mystery => "Something stirs in the dark...",
            _ => "",
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                headline_text,
                Style::default().fg(Color::Gray),
            )))
            .alignment(Alignment::Center),
            chunks[0],
        );

        let body_lines: Vec<Line> = match self.kind {
            NodeKind::Camp => {
                let mut lines = Vec::new();
                if let Some(hp) = self.healed_to {
                    lines.push(Line::from(Span::styled(
                        format!("HP and MP restored to full ({} HP).", hp),
                        Style::default().fg(Color::Rgb(180, 220, 130)),
                    )));
                }
                lines
            }
            NodeKind::Shop | NodeKind::Mystery => {
                vec![Line::from(Span::styled(
                    "(coming soon)",
                    Style::default().fg(Color::DarkGray),
                ))]
            }
            _ => vec![],
        };
        frame.render_widget(
            Paragraph::new(body_lines).alignment(Alignment::Center),
            chunks[1],
        );

        let hint = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::styled(" continue", Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(
            Paragraph::new(hint).alignment(Alignment::Center),
            chunks[2],
        );
    }
}
