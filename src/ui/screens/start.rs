use crate::data::starters;
use crate::map;
use crate::meta;
use crate::player::Player;
use crate::run::Run;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::settings::SettingsOrigin;
use crate::ui::screens::{MapScreen, SettingsScreen, ShopScreen, StarterSelectScreen};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartChoice {
    Play,
    Shop,
    Settings,
}

impl StartChoice {
    const ALL: &'static [StartChoice] = &[
        StartChoice::Play,
        StartChoice::Shop,
        StartChoice::Settings,
    ];

    fn label(&self) -> &'static str {
        match self {
            StartChoice::Play => "Play",
            StartChoice::Shop => "Shop",
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

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
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
                StartChoice::Play => start_play(player),
                StartChoice::Shop => Transition::Goto(Screen::Shop(ShopScreen::new())),
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
        let menu_w: u16 = 32;
        let menu_h: u16 = (StartChoice::ALL.len() as u16) * 2 + 2;
        let tagline_h: u16 = 1;
        let balance_h: u16 = 1;
        let hint_h: u16 = 1;
        let gap: u16 = 1;

        let total_h =
            title_h + gap + tagline_h + gap + balance_h + gap + menu_h + gap + hint_h;
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
            "A roguelike dungeon crawler",
            Style::default().fg(Color::Gray),
        );
        y += gap;

        render_balance(frame, area, &mut y);
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

fn start_play(player: &mut Player) -> Transition {
    // First-launch path: nobody owned yet, send the player to pick.
    if !meta::has_any_monster() {
        return Transition::Goto(Screen::StarterSelect(StarterSelectScreen::new()));
    }
    // Otherwise build a run from the active party member's species.
    let Some(species) = meta::active_party_species() else {
        return Transition::Goto(Screen::StarterSelect(StarterSelectScreen::new()));
    };
    let Some(starter) = starters::all_starters()
        .into_iter()
        .find(|s| s.name == species)
    else {
        // Stored species no longer matches a known starter (renamed?
        // removed?) — fall back to the picker.
        return Transition::Goto(Screen::StarterSelect(StarterSelectScreen::new()));
    };
    *player = Player::for_starter(&starter);
    let id = meta::starter_id(&starter.name);
    let party = vec![crate::run::PartyMember::from_starter(id, starter)];
    let run = Run::new(party, map::generate());
    Transition::Goto(Screen::Map(MapScreen::with_run(run)))
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

fn render_balance(frame: &mut Frame, area: Rect, y: &mut u16) {
    if *y >= area.y + area.height {
        return;
    }
    let row = Rect {
        x: area.x,
        y: *y,
        width: area.width,
        height: 1,
    };
    let snap = crate::meta::snapshot();
    let line = Line::from(vec![
        Span::styled("Embers ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", snap.embers),
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), row);
    *y += 1;
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
