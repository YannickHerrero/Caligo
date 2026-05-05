use crate::data::starters::{Starter, StarterVisual};
use crate::meta::{self, MonsterId, StarterRanks, Upgrade};
use crate::player::Player;
use crate::run::PartyMember;
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

/// Cross-run roster manager. Lists every owned monster, lets the player
/// toggle which ones are in the active party (max 6, min 1), and shows
/// a detail card for the highlighted monster: visual, type, stats with
/// permanent-upgrade bonuses applied, per-ladder rank breakdown,
/// starting moves, and description.
pub struct CollectionScreen {
    selected: usize,
    message: Option<String>,
}

impl CollectionScreen {
    pub fn new() -> Self {
        Self {
            selected: 0,
            message: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        let owned = sorted_owned_ids();
        let len = owned.len();
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                Transition::Goto(Screen::Start(StartScreen::new()))
            }
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
                if let Some(id) = owned.get(self.selected) {
                    match meta::toggle_party(id) {
                        Ok(_) => self.message = None,
                        Err(e) => self.message = Some(e),
                    }
                }
                Transition::Stay
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
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title
                Constraint::Length(1), // party count summary
                Constraint::Length(1), // spacer
                Constraint::Min(8),    // body (split)
                Constraint::Length(1), // message
                Constraint::Length(1), // hint
            ])
            .split(area);

        let snap = meta::snapshot();
        render_title(frame, chunks[0]);
        render_party_count(frame, &snap, chunks[1]);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[3]);
        render_owned_list(frame, &snap, self.selected, body[0]);
        render_detail(frame, &snap, self.selected, body[1]);

        if let Some(msg) = self.message.as_deref() {
            render_message(frame, msg, chunks[4]);
        }
        render_hint(frame, chunks[5]);
    }
}

fn sorted_owned_ids() -> Vec<MonsterId> {
    let snap = meta::snapshot();
    let mut ids: Vec<MonsterId> = snap.monsters.keys().cloned().collect();
    ids.sort();
    ids
}

fn render_title(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        "Caligo \u{2014} Collection",
        Style::default()
            .fg(Color::Rgb(255, 140, 90))
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_party_count(frame: &mut Frame, snap: &meta::Meta, area: Rect) {
    let line = Line::from(vec![
        Span::styled("Party ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}/{}", snap.party.len(), meta::PARTY_CAP),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   \u{00B7}   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Owned ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", snap.monsters.len()),
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_owned_list(
    frame: &mut Frame,
    snap: &meta::Meta,
    selected: usize,
    area: Rect,
) {
    let owned = sorted_owned_ids();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Owned Monsters ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    if owned.is_empty() {
        let line = Line::from(Span::styled(
            "(no monsters owned yet)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            inner,
        );
        return;
    }

    // Visible window so long collections can scroll. Each entry is one
    // line; cursor stays roughly mid-window.
    let visible = inner.height as usize;
    let scroll = if owned.len() <= visible {
        0
    } else {
        let half = visible / 2;
        if selected < half {
            0
        } else if selected + (visible - half) >= owned.len() {
            owned.len() - visible
        } else {
            selected - half
        }
    };
    let end = (scroll + visible).min(owned.len());

    let mut lines: Vec<Line> = Vec::new();
    for (offset, id) in owned[scroll..end].iter().enumerate() {
        let idx = scroll + offset;
        let is_selected = idx == selected;
        let cursor = if is_selected { "\u{25B6} " } else { "  " };
        let in_party = snap.party.iter().any(|p| p == id);
        let mark = if in_party { "[\u{2713}]" } else { "[ ]" };
        let header_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let mark_style = if in_party {
            Style::default()
                .fg(Color::Rgb(180, 220, 130))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let species = snap
            .monsters
            .get(id)
            .map(|m| m.species.clone())
            .unwrap_or_else(|| id.clone());
        lines.push(Line::from(vec![
            Span::styled(cursor, header_style),
            Span::styled(format!("{} ", mark), mark_style),
            Span::styled(species, header_style),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_detail(
    frame: &mut Frame,
    snap: &meta::Meta,
    selected: usize,
    area: Rect,
) {
    let owned = sorted_owned_ids();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Details ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let Some(id) = owned.get(selected) else {
        let line = Line::from(Span::styled(
            "(no monster selected)",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            inner,
        );
        return;
    };
    let Some(instance) = snap.monsters.get(id) else {
        return;
    };
    let Some(member) = crate::run::build_party_member_from_instance(instance) else {
        return;
    };
    let template = &member.template;
    let ranks = snap.monster_ranks.get(id).copied().unwrap_or_default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // name + type chip
            Constraint::Length(8),  // visual
            Constraint::Length(2),  // stats
            Constraint::Length(5),  // 4 ladder rows
            Constraint::Length(2),  // moves
            Constraint::Min(1),     // description
        ])
        .split(inner);

    render_name_row(frame, template, chunks[0]);
    render_visual(frame, template, chunks[1]);
    render_stats(frame, &member, ranks, chunks[2]);
    render_ranks(frame, ranks, chunks[3]);
    render_moves(frame, template, chunks[4]);
    render_description(frame, template, chunks[5]);
}

fn render_name_row(frame: &mut Frame, template: &Starter, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            template.name.clone(),
            Style::default()
                .fg(template.primary_type.color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ", Style::default()),
        Span::styled(
            format!("({})", template.primary_type.label()),
            Style::default().fg(template.primary_type.color()),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_visual(frame: &mut Frame, template: &Starter, area: Rect) {
    let color = template.color();
    let lines: Vec<Line> = match &template.visual {
        StarterVisual::AnimatedCrab => {
            // Static idle frame. The Catalogue runs a live Crab entity
            // for animation; here we just show a snapshot.
            const FRAME: &[&str] = &[
                "    _~^~^~_     ",
                "\\) /  o   o  \\ (/",
                "  '_   ---   _'  ",
                "  \\ '-------' /  ",
            ];
            FRAME
                .iter()
                .map(|row| {
                    Line::from(Span::styled(
                        (*row).to_string(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ))
                })
                .collect()
        }
        StarterVisual::Frames(frames) => {
            if let Some(frame_lines) = frames.first() {
                frame_lines
                    .iter()
                    .map(|row| {
                        Line::from(Span::styled(
                            row.clone(),
                            Style::default().fg(color).add_modifier(Modifier::BOLD),
                        ))
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
    };
    let h = lines.len() as u16;
    let pad = area.height.saturating_sub(h) / 2;
    let mut padded: Vec<Line> = Vec::with_capacity(lines.len() + pad as usize);
    for _ in 0..pad {
        padded.push(Line::from(""));
    }
    padded.extend(lines);
    frame.render_widget(
        Paragraph::new(padded).alignment(Alignment::Center),
        area,
    );
}

fn render_stats(
    frame: &mut Frame,
    member: &PartyMember,
    ranks: StarterRanks,
    area: Rect,
) {
    // Two-column view: HP/MP on top, Speed/Atk on bottom. Each value
    // shows the bonus from ladder ranks in dim text.
    let hp_bonus = ranks.tidepool * 2;
    let mp_bonus = ranks.wellspring;
    let speed_bonus = ranks.quickfoot;
    let atk_pct = (member.attack_boost_pct * 100.0).round() as u32;

    let dim = Style::default().fg(Color::DarkGray);
    let line1 = Line::from(vec![
        Span::styled("HP ", dim),
        Span::styled(
            format!("{}", member.max_hp),
            Style::default()
                .fg(Color::Rgb(255, 120, 80))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" (+{})    ", hp_bonus), dim),
        Span::styled("MP ", dim),
        Span::styled(
            format!("{}", member.max_mana),
            Style::default()
                .fg(Color::Rgb(120, 160, 255))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" (+{})", mp_bonus), dim),
    ]);
    let line2 = Line::from(vec![
        Span::styled("Speed ", dim),
        Span::styled(
            format!("{}", member.speed),
            Style::default()
                .fg(Color::Rgb(180, 220, 200))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" (+{})    ", speed_bonus), dim),
        Span::styled("Atk ", dim),
        Span::styled(
            format!("+{}%", atk_pct),
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let widget = Paragraph::new(vec![line1, line2]).alignment(Alignment::Center);
    frame.render_widget(widget, area);
}

fn render_ranks(frame: &mut Frame, ranks: StarterRanks, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    for upgrade in Upgrade::ALL {
        let current = upgrade.current_rank(&ranks);
        let max = upgrade.max_rank();
        let bar = rank_bar(current, max);
        let label_style = Style::default().fg(Color::Gray);
        let bar_style = if current == max {
            Style::default()
                .fg(Color::Rgb(120, 200, 120))
                .add_modifier(Modifier::BOLD)
        } else if current > 0 {
            Style::default().fg(Color::Rgb(255, 140, 90))
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{:<18}", upgrade.name()), label_style),
            Span::styled(bar, bar_style),
            Span::styled(
                format!("  {}/{}", current, max),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}

fn rank_bar(current: u32, max: u32) -> String {
    let max = max.max(1);
    let mut s = String::new();
    for i in 0..max {
        if i < current {
            s.push('\u{25A0}'); // ■
        } else {
            s.push('\u{25A1}'); // □
        }
    }
    s
}

fn render_moves(frame: &mut Frame, template: &Starter, area: Rect) {
    let dim = Style::default().fg(Color::DarkGray);
    let line = Line::from(vec![
        Span::styled("Moves  ", dim),
        Span::styled(
            template.starting_attacks.join(", "),
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_description(frame: &mut Frame, template: &Starter, area: Rect) {
    let line = Line::from(Span::styled(
        template.description.clone(),
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
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
        Span::styled(" toggle party   ", dim),
        Span::styled("Esc", key),
        Span::styled(" back", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
