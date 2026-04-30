use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::settings::SettingsOrigin;
use crate::ui::screens::{MapScreen, SettingsScreen};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const TITLE_ART: &[&str] = &[
    " ██████╗ █████╗ ██╗     ██╗ ██████╗  ██████╗ ",
    "██╔════╝██╔══██╗██║     ██║██╔════╝ ██╔═══██╗",
    "██║     ███████║██║     ██║██║  ███╗██║   ██║",
    "██║     ██╔══██║██║     ██║██║   ██║██║   ██║",
    "╚██████╗██║  ██║███████╗██║╚██████╔╝╚██████╔╝",
    " ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝ ╚═════╝  ╚═════╝ ",
];

const CRAB_ART: &[&str] = &[
    "    _~^~^~_    ",
    "\\) /  o   o  \\ (/",
    "  '_   ---   _'  ",
    "  \\ '-------' /  ",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartChoice {
    Play,
    Settings,
}

impl StartChoice {
    const ALL: &'static [StartChoice] = &[StartChoice::Play, StartChoice::Settings];

    fn label(&self) -> &'static str {
        match self {
            StartChoice::Play => "Play",
            StartChoice::Settings => "Settings",
        }
    }
}

pub struct StartScreen {
    pub selected: usize,
}

impl StartScreen {
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        let len = StartChoice::ALL.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => Transition::Quit,
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = (self.selected + len - 1) % len;
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % len;
                Transition::Stay
            }
            KeyCode::Enter => match StartChoice::ALL[self.selected] {
                StartChoice::Play => Transition::Goto(Screen::Map(MapScreen::new())),
                StartChoice::Settings => Transition::Goto(Screen::Settings(
                    SettingsScreen::new(SettingsOrigin::Start),
                )),
            },
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

        let title_w = TITLE_ART.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        let title_h = TITLE_ART.len() as u16;
        let crab_w = CRAB_ART.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
        let crab_h = CRAB_ART.len() as u16;
        let menu_w: u16 = 32;
        let menu_h: u16 = (StartChoice::ALL.len() as u16) * 2 + 2;
        let tagline_h: u16 = 1;
        let hint_h: u16 = 1;
        let gap: u16 = 1;

        let total_h = title_h + gap + tagline_h + gap + crab_h + gap + menu_h + gap + hint_h;
        let mut y = area.y + area.height.saturating_sub(total_h) / 2;

        render_centered_lines(
            frame,
            area,
            &mut y,
            TITLE_ART,
            title_w,
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        );
        y += gap;

        render_centered_text(
            frame,
            area,
            &mut y,
            "A roguelike crab dungeon crawler",
            Style::default().fg(Color::Gray),
        );
        y += gap;

        render_centered_lines(
            frame,
            area,
            &mut y,
            CRAB_ART,
            crab_w,
            Style::default()
                .fg(Color::Rgb(220, 110, 90))
                .add_modifier(Modifier::BOLD),
        );
        y += gap;

        render_menu(frame, area, &mut y, menu_w, menu_h, self.selected);
        y += gap;

        render_centered_text(
            frame,
            area,
            &mut y,
            "↑↓ navigate · Enter select · q quit",
            Style::default().fg(Color::DarkGray),
        );
    }
}

fn render_centered_lines(
    frame: &mut Frame,
    area: Rect,
    y: &mut u16,
    lines: &[&str],
    width: u16,
    style: Style,
) {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let height = lines.len() as u16;
    if *y + height > area.y + area.height {
        return;
    }
    let block_area = Rect {
        x,
        y: *y,
        width,
        height,
    };
    let lines: Vec<Line> = lines
        .iter()
        .map(|l| Line::from(Span::styled((*l).to_string(), style)))
        .collect();
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), block_area);
    *y += height;
}

fn render_centered_text(frame: &mut Frame, area: Rect, y: &mut u16, text: &str, style: Style) {
    if *y >= area.y + area.height {
        return;
    }
    let row = Rect {
        x: area.x,
        y: *y,
        width: area.width,
        height: 1,
    };
    let line = Line::from(Span::styled(text.to_string(), style));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), row);
    *y += 1;
}

fn render_menu(frame: &mut Frame, area: Rect, y: &mut u16, width: u16, height: u16, selected: usize) {
    let width = width.min(area.width);
    let x = area.x + area.width.saturating_sub(width) / 2;
    if *y + height > area.y + area.height {
        return;
    }
    let menu_area = Rect {
        x,
        y: *y,
        width,
        height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Menu ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);

    let mut lines: Vec<Line> = Vec::new();
    for (idx, choice) in StartChoice::ALL.iter().enumerate() {
        let is_selected = idx == selected;
        let cursor = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(vec![
            Span::styled(cursor, style),
            Span::styled(choice.label(), style),
        ]));
        lines.push(Line::from(""));
    }
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        inner,
    );
    *y += height;
}
