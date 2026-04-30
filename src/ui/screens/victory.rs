use crate::data::starters::Starter;
use crate::fight::Item;
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::StartScreen;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Shown when the player defeats the dungeon's boss. Recaps the run and
/// routes back to the start menu.
pub struct VictoryScreen {
    pub starter: Starter,
    pub floor_reached: u32,
    pub gold_total: u32,
    pub boss_gold: u32,
    pub boss_items: Vec<Item>,
}

impl VictoryScreen {
    pub fn new(
        starter: Starter,
        floor_reached: u32,
        gold_total: u32,
        boss_gold: u32,
        boss_items: Vec<Item>,
    ) -> Self {
        Self {
            starter,
            floor_reached,
            gold_total,
            boss_gold,
            boss_items,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        match key {
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('q') | KeyCode::Esc => {
                Transition::Goto(Screen::Start(StartScreen::new()))
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

        let panel_w = 56.min(area.width.saturating_sub(4)).max(20);
        let panel_h = (8 + self.boss_items.len() as u16 + 4).min(area.height.saturating_sub(2));
        let panel = Rect {
            x: area.x + (area.width.saturating_sub(panel_w)) / 2,
            y: area.y + (area.height.saturating_sub(panel_h)) / 2,
            width: panel_w,
            height: panel_h,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(255, 200, 80)))
            .title(Span::styled(
                " ★ Run Cleared ★ ",
                Style::default()
                    .fg(Color::Rgb(255, 220, 100))
                    .add_modifier(Modifier::BOLD),
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
                Constraint::Length(2), // boss reward gold
                Constraint::Min(1),    // boss item drops
                Constraint::Length(1), // separator
                Constraint::Length(1), // starter
                Constraint::Length(1), // floor
                Constraint::Length(1), // gold total
                Constraint::Length(1), // hint
            ])
            .split(inner);

        let headline = Line::from(Span::styled(
            "The boss falls. The dungeon stills.",
            Style::default().fg(Color::Gray),
        ));
        frame.render_widget(
            Paragraph::new(headline).alignment(Alignment::Center),
            chunks[0],
        );

        let boss_gold = Line::from(vec![
            Span::styled(
                format!("+{}", self.boss_gold),
                Style::default()
                    .fg(Color::Rgb(240, 210, 110))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" gold", Style::default().fg(Color::Gray)),
        ]);
        frame.render_widget(
            Paragraph::new(boss_gold).alignment(Alignment::Center),
            chunks[1],
        );

        let item_lines: Vec<Line> = if self.boss_items.is_empty() {
            vec![Line::from(Span::styled(
                "(no items)",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            self.boss_items
                .iter()
                .map(|item| {
                    Line::from(vec![
                        Span::styled(
                            "+ ",
                            Style::default().fg(Color::Rgb(180, 220, 130)),
                        ),
                        Span::styled(
                            item.name(),
                            Style::default()
                                .fg(item.color())
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                })
                .collect()
        };
        frame.render_widget(
            Paragraph::new(item_lines).alignment(Alignment::Center),
            chunks[2],
        );

        let separator = Line::from(Span::styled(
            "·  ·  ·",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(separator).alignment(Alignment::Center),
            chunks[3],
        );

        let starter_line = Line::from(vec![
            Span::styled("Starter ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                self.starter.name.clone(),
                Style::default()
                    .fg(self.starter.primary_type.color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(starter_line).alignment(Alignment::Center),
            chunks[4],
        );

        let floor_line = Line::from(vec![
            Span::styled("Floor   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", self.floor_reached),
                Style::default().fg(Color::Gray),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(floor_line).alignment(Alignment::Center),
            chunks[5],
        );

        let gold_total_line = Line::from(vec![
            Span::styled("Gold    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", self.gold_total),
                Style::default().fg(Color::Rgb(240, 210, 110)),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(gold_total_line).alignment(Alignment::Center),
            chunks[6],
        );

        let hint = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::styled(" return to title", Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(
            Paragraph::new(hint).alignment(Alignment::Center),
            chunks[7],
        );
    }
}
