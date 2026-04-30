use crate::data::starters::Starter;
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

pub struct GameOverScreen {
    pub starter: Starter,
    pub floor_reached: u32,
    pub gold_earned: u32,
}

impl GameOverScreen {
    pub fn new(starter: Starter, floor_reached: u32, gold_earned: u32) -> Self {
        Self {
            starter,
            floor_reached,
            gold_earned,
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

        let panel_w = 50.min(area.width.saturating_sub(4)).max(20);
        let panel_h: u16 = 11.min(area.height.saturating_sub(2));
        let panel = Rect {
            x: area.x + (area.width.saturating_sub(panel_w)) / 2,
            y: area.y + (area.height.saturating_sub(panel_h)) / 2,
            width: panel_w,
            height: panel_h,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(180, 60, 60)))
            .title(Span::styled(
                " You Died ",
                Style::default()
                    .fg(Color::Rgb(220, 80, 80))
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
                Constraint::Length(1), // starter
                Constraint::Length(1), // floor
                Constraint::Length(1), // gold
                Constraint::Min(1),    // spacer
                Constraint::Length(1), // hint
            ])
            .split(inner);

        let headline = Line::from(Span::styled(
            format!("Run ended on floor {}.", self.floor_reached),
            Style::default().fg(Color::Gray),
        ));
        frame.render_widget(
            Paragraph::new(headline).alignment(Alignment::Center),
            chunks[0],
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
            chunks[1],
        );

        let gold_line = Line::from(vec![
            Span::styled("Gold    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", self.gold_earned),
                Style::default().fg(Color::Rgb(240, 210, 110)),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(gold_line).alignment(Alignment::Center),
            chunks[3],
        );

        let hint = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::styled(" return to title", Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(
            Paragraph::new(hint).alignment(Alignment::Center),
            chunks[5],
        );
    }
}
