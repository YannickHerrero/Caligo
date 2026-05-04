use crate::crab::Crab;
use crate::data::starters::{self, Starter, StarterVisual};
use crate::map;
use crate::player::Player;
use crate::run::Run;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::{MapScreen, StartScreen};
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const FRAME_TICK: f32 = 0.05;
const FRAME_DURATION: f32 = 0.2;

pub struct StarterSelectScreen {
    starters: Vec<Starter>,
    selected: usize,
    crab: Crab,
    crab_bounds: (f32, f32),
    frame_timer: f32,
    frame_index: usize,
}

impl StarterSelectScreen {
    pub fn new() -> Self {
        let mut crab = Crab::new((0.0, 0.0), 95);
        crab.walk_range_x = Some((0.0, 0.5));
        Self {
            starters: starters::all_starters(),
            selected: 0,
            crab,
            crab_bounds: (0.0, 0.0),
            frame_timer: 0.0,
            frame_index: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, player: &mut Player) -> Transition {
        let len = self.starters.len();
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                Transition::Goto(Screen::Start(StartScreen::new()))
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if len > 0 {
                    self.selected = (self.selected + len - 1) % len;
                }
                Transition::Stay
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if len > 0 {
                    self.selected = (self.selected + 1) % len;
                }
                Transition::Stay
            }
            KeyCode::Enter => {
                let Some(starter) = self.starters.get(self.selected).cloned() else {
                    return Transition::Stay;
                };
                // Persist the pick: the player permanently owns this
                // starter and joins it as their first party member.
                crate::meta::add_owned_starter(&starter.name);
                *player = Player::for_starter(&starter);
                let id = crate::meta::starter_id(&starter.name);
                let party = vec![crate::run::PartyMember::from_starter(id, starter)];
                let run = Run::new(party, map::generate());
                Transition::Goto(Screen::Map(MapScreen::with_run(run)))
            }
            _ => Transition::Stay,
        }
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        if self.crab_bounds.0 > 0.0 && self.crab_bounds.1 > 0.0 {
            self.crab.update(FRAME_TICK, self.crab_bounds);
        }
        self.frame_timer += FRAME_TICK;
        if self.frame_timer >= FRAME_DURATION {
            self.frame_timer -= FRAME_DURATION;
            self.frame_index = self.frame_index.wrapping_add(1);
        }
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
        let area = frame.area();
        if area.width == 0 || area.height == 0 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(1),
            ])
            .split(area);
        let title_area = chunks[0];
        let subtitle_area = chunks[1];
        let body_area = chunks[2];
        let hint_area = chunks[3];

        render_title(frame, title_area);
        render_subtitle(frame, subtitle_area);

        let card_count = self.starters.len() as u16;
        if card_count == 0 {
            return;
        }
        let card_constraints: Vec<Constraint> = (0..card_count)
            .map(|_| Constraint::Ratio(1, card_count as u32))
            .collect();
        let card_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(card_constraints)
            .split(body_area);

        for idx in 0..self.starters.len() {
            let card_area = card_areas[idx];
            let is_selected = idx == self.selected;
            self.update_crab_bounds(card_area, is_selected);
            let starter = &self.starters[idx];
            render_card(
                frame,
                starter,
                is_selected,
                &self.crab,
                self.frame_index,
                card_area,
            );
        }

        render_hint(frame, hint_area);
    }

    fn update_crab_bounds(&mut self, card_area: Rect, is_selected: bool) {
        if !is_selected {
            return;
        }
        // Match the inner sprite-area inside a card so the crab stays
        // centered relative to where its frames render.
        let inner_w = card_area.width.saturating_sub(2);
        let inner_h = card_area.height.saturating_sub(2);
        let sprite_h = inner_h.saturating_sub(8); // type chip + meta lines
        let bounds = (inner_w as f32, sprite_h as f32);
        if bounds == self.crab_bounds || bounds.0 <= 0.0 || bounds.1 <= 0.0 {
            return;
        }
        let cx = ((bounds.0 - 15.0).max(0.0)) / 2.0;
        self.crab.position.0 = cx;
        self.crab.walk_range_x = Some((cx, cx + 0.5));
        self.crab_bounds = bounds;
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        "Caligo — Choose your starter",
        Style::default()
            .fg(Color::Rgb(255, 140, 90))
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_subtitle(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        "Pick a companion. They'll bring their own four moves.",
        Style::default().fg(Color::Gray),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_hint(frame: &mut Frame, area: Rect) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let hint = Line::from(vec![
        Span::styled("← →", key),
        Span::styled(" navigate   ", dim),
        Span::styled("Enter", key),
        Span::styled(" choose   ", dim),
        Span::styled("Esc", key),
        Span::styled(" back", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}

fn render_card(
    frame: &mut Frame,
    starter: &Starter,
    is_selected: bool,
    crab: &Crab,
    frame_index: usize,
    area: Rect,
) {
    let border_color = if is_selected {
        starter.primary_type.color()
    } else {
        Color::DarkGray
    };
    let title_style = if is_selected {
        Style::default()
            .fg(starter.primary_type.color())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let title_text = if is_selected {
        format!(" ▶ {} ", starter.name)
    } else {
        format!("   {}   ", starter.name)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(title_text, title_style));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // type chip
            Constraint::Min(4),    // sprite
            Constraint::Length(2), // moves
            Constraint::Length(3), // description
        ])
        .split(inner);
    let chip_area = chunks[0];
    let sprite_area = chunks[1];
    let moves_area = chunks[2];
    let desc_area = chunks[3];

    let chip = Line::from(Span::styled(
        starter.primary_type.label().to_string(),
        Style::default()
            .fg(starter.primary_type.color())
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(
        Paragraph::new(chip).alignment(Alignment::Center),
        chip_area,
    );

    match &starter.visual {
        StarterVisual::AnimatedCrab => {
            // Only the selected card animates the live crab (it's a single
            // shared Crab entity); other cards use a static fallback frame.
            if is_selected {
                widgets::render_crab(frame, crab, sprite_area, None);
            } else {
                render_static_crab(frame, starter.color(), sprite_area);
            }
        }
        StarterVisual::Frames(frames) => {
            if frames.is_empty() {
                return;
            }
            let sprite = &frames[frame_index % frames.len()];
            let lines: Vec<Line> = sprite
                .iter()
                .map(|row| {
                    Line::from(Span::styled(
                        row.clone(),
                        Style::default()
                            .fg(starter.color())
                            .add_modifier(Modifier::BOLD),
                    ))
                })
                .collect();
            let pad = sprite_area
                .height
                .saturating_sub(sprite.len() as u16)
                / 2;
            let mut padded: Vec<Line> = Vec::with_capacity(lines.len() + pad as usize);
            for _ in 0..pad {
                padded.push(Line::from(""));
            }
            padded.extend(lines);
            frame.render_widget(
                Paragraph::new(padded).alignment(Alignment::Center),
                sprite_area,
            );
        }
    }

    let moves_line = Line::from(Span::styled(
        starter.starting_attacks.join(" · "),
        Style::default().fg(Color::Gray),
    ));
    frame.render_widget(
        Paragraph::new(moves_line).alignment(Alignment::Center),
        moves_area,
    );

    let desc_line = Line::from(Span::styled(
        starter.description.clone(),
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(
        Paragraph::new(desc_line).alignment(Alignment::Center),
        desc_area,
    );
}

/// A simple fallback for non-selected AnimatedCrab cards so they don't
/// pull from the live Crab entity that the selected card animates.
fn render_static_crab(frame: &mut Frame, color: Color, area: Rect) {
    let sprite = [
        "    _~^~^~_   ",
        "\\) /  o o  \\ (/",
        "  '_   -   _'  ",
        "  \\ '-----' /  ",
    ];
    let pad = area.height.saturating_sub(sprite.len() as u16) / 2;
    let mut lines: Vec<Line> = Vec::with_capacity(sprite.len() + pad as usize);
    for _ in 0..pad {
        lines.push(Line::from(""));
    }
    for row in sprite {
        lines.push(Line::from(Span::styled(
            row.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}
