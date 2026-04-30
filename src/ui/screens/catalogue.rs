use crate::data::attacks as attack_lib;
use crate::fight::{
    impact_for, trail_for, AnimationKind, Attack, Effect, ParticleKind, ProjectileKind,
    ProjectileSize,
};
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::SelectScreen;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CatalogueScreen {
    pub attacks: Vec<Attack>,
    pub selected: usize,
    pub scroll: usize,
    last_list_height: u16,
}

impl CatalogueScreen {
    pub fn new() -> Self {
        Self {
            attacks: attack_lib::all_attacks(),
            selected: 0,
            scroll: 0,
            last_list_height: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        let len = self.attacks.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                Transition::Goto(Screen::Select(SelectScreen::new()))
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.scroll {
                        self.scroll = self.selected;
                    }
                }
                Transition::Stay
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < len {
                    self.selected += 1;
                    let visible = self.last_list_height as usize;
                    if visible > 0 && self.selected >= self.scroll + visible {
                        self.scroll = self.selected + 1 - visible;
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
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(1),
            ])
            .split(area);
        let header_area = v_chunks[0];
        let main_area = v_chunks[1];
        let hint_area = v_chunks[2];

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
            .split(main_area);
        let list_area = main_chunks[0];
        let right_area = main_chunks[1];

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(right_area);
        let visuals_area = right_chunks[0];
        let info_area = right_chunks[1];

        render_header(frame, header_area);

        self.last_list_height = list_area.height.saturating_sub(2);
        render_list(
            frame,
            &self.attacks,
            self.selected,
            self.scroll,
            list_area,
        );

        if let Some(attack) = self.attacks.get(self.selected) {
            render_visuals(frame, attack, visuals_area);
            render_info(frame, attack, info_area);
        }

        render_hint(frame, hint_area);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            "Caligo — Catalogue",
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ·   ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "static projectile and particle previews",
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_hint(frame: &mut Frame, area: Rect) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let hint = Line::from(vec![
        Span::styled("↑ ↓", key),
        Span::styled(" scroll   ", dim),
        Span::styled("q", key),
        Span::styled(" back", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}

fn render_list(
    frame: &mut Frame,
    attacks: &[Attack],
    selected: usize,
    scroll: usize,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Attacks ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible = inner.height as usize;
    if visible == 0 {
        return;
    }
    let end = (scroll + visible).min(attacks.len());

    let mut lines: Vec<Line> = Vec::with_capacity(end - scroll);
    for (offset, attack) in attacks[scroll..end].iter().enumerate() {
        let idx = scroll + offset;
        let is_selected = idx == selected;
        let cursor = if is_selected { "▶ " } else { "  " };
        let name_style = if is_selected {
            Style::default()
                .fg(attack.element.color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(vec![
            Span::styled(cursor, name_style),
            Span::styled(attack.name.clone(), name_style),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_info(frame: &mut Frame, attack: &Attack, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Info ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        attack.name.clone(),
        Style::default()
            .fg(attack.element.color())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Effect    ", Style::default().fg(Color::DarkGray)),
        Span::styled(attack.effect.label(), Style::default().fg(attack.effect.color())),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Mana      ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", attack.mana_cost),
            Style::default().fg(Color::Rgb(120, 160, 255)),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Type      ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            attack.element.label().to_string(),
            Style::default().fg(attack.element.color()),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Animation ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            animation_label(attack),
            Style::default().fg(Color::Rgb(180, 180, 180)),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        attack.description.clone(),
        Style::default().fg(Color::Gray),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn animation_label(attack: &Attack) -> String {
    match attack.kind {
        AnimationKind::Jump => "Jump".to_string(),
        AnimationKind::Dash => "Dash".to_string(),
        AnimationKind::Throw(p) => format!("Throw({:?})", p),
        AnimationKind::SelfCast(p) => format!("SelfCast({:?})", p),
    }
}

fn render_visuals(frame: &mut Frame, attack: &Attack, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Visuals ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let projectile = projectile_for(attack);
    let trail = trail_for(attack.kind, attack.element);
    let impact = impact_for(attack.kind, attack.element, &attack.effect);
    let aura = match attack.kind {
        AnimationKind::SelfCast(p) => Some(p),
        _ => None,
    };

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(inner);
    render_projectile_panel(frame, projectile, halves[0]);
    render_particle_panel(frame, trail, impact, aura, halves[1]);
}

fn projectile_for(attack: &Attack) -> Option<(ProjectileKind, ProjectileSize)> {
    let kind = match attack.kind {
        AnimationKind::Throw(k) => k,
        _ => return None,
    };
    let size = match attack.effect {
        Effect::Damage(d) => ProjectileSize::for_damage(d),
        _ => ProjectileSize::Small,
    };
    Some((kind, size))
}

fn render_projectile_panel(
    frame: &mut Frame,
    projectile: Option<(ProjectileKind, ProjectileSize)>,
    area: Rect,
) {
    if area.width < 4 || area.height < 3 {
        return;
    }
    let title = Line::from(Span::styled(
        " Projectile ",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::UNDERLINED),
    ));
    let title_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    frame.render_widget(Paragraph::new(title).alignment(Alignment::Center), title_area);

    let body_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height.saturating_sub(1),
    };

    let Some((kind, size)) = projectile else {
        let line = Line::from(Span::styled("—", Style::default().fg(Color::DarkGray)));
        frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), body_area);
        return;
    };

    let sprite = kind.sprite(size);
    let label = format!("{:?} ({:?})", kind, size);
    let mut lines: Vec<Line> = Vec::with_capacity(sprite.len() + 2);
    let pad = body_area
        .height
        .saturating_sub((sprite.len() + 2) as u16)
        / 2;
    for _ in 0..pad {
        lines.push(Line::from(""));
    }
    for row in sprite {
        lines.push(Line::from(Span::styled(
            row.to_string(),
            Style::default().fg(kind.color()),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        label,
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        body_area,
    );
}

fn render_particle_panel(
    frame: &mut Frame,
    trail: Option<ParticleKind>,
    impact: Option<ParticleKind>,
    aura: Option<ParticleKind>,
    area: Rect,
) {
    if area.width < 4 || area.height < 3 {
        return;
    }
    let title = Line::from(Span::styled(
        " Particles ",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::UNDERLINED),
    ));
    let title_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    frame.render_widget(Paragraph::new(title).alignment(Alignment::Center), title_area);

    let mut entries: Vec<(&'static str, ParticleKind)> = Vec::new();
    if let Some(p) = aura {
        entries.push(("Aura", p));
    }
    if let Some(p) = trail {
        entries.push(("Trail", p));
    }
    if let Some(p) = impact {
        entries.push(("Impact", p));
    }

    if entries.is_empty() {
        let body = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(1),
        };
        let line = Line::from(Span::styled("—", Style::default().fg(Color::DarkGray)));
        frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), body);
        return;
    }

    let body = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height.saturating_sub(1),
    };
    let rows = entries.len() as u16;
    let constraints: Vec<Constraint> = (0..rows)
        .map(|_| Constraint::Ratio(1, rows as u32))
        .collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(body);
    for (idx, (label, kind)) in entries.into_iter().enumerate() {
        render_particle_row(frame, label, kind, chunks[idx]);
    }
}

fn render_particle_row(frame: &mut Frame, label: &str, kind: ParticleKind, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let glyph = kind.glyph();
    let style = Style::default().fg(kind.color());
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(format!("{:<7}", label), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:?}", kind), Style::default().fg(Color::Gray)),
    ]));
    let pattern = particle_pattern(glyph);
    for row in pattern {
        lines.push(Line::from(Span::styled(row, style)));
    }
    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        area,
    );
}

fn particle_pattern(glyph: &str) -> Vec<String> {
    let g = glyph;
    vec![
        format!("{}   {}   {}", g, g, g),
        format!("  {}   {}", g, g),
        format!("{}   {}   {}", g, g, g),
    ]
}
