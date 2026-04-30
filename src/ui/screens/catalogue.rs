use crate::crab::Crab;
use crate::data::attacks as attack_lib;
use crate::data::enemies as enemy_lib;
use crate::data::starters as starter_lib;
use crate::data::starters::{Starter, StarterVisual};
use crate::environment::{Environment, GroundStyle, TimeOfDay};
use crate::fight::{
    impact_for, trail_for, AnimationKind, Attack, Effect, Element, Enemy, ParticleKind,
    ProjectileKind, ProjectileSize,
};
use crate::player::Player;
use crate::ui::screen::{Screen, Transition};
use crate::ui::screens::SelectScreen;
use crate::ui::widgets;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogueTab {
    Attacks,
    Environments,
    Bestiary,
    Starters,
}

impl CatalogueTab {
    const ALL: &'static [CatalogueTab] = &[
        CatalogueTab::Attacks,
        CatalogueTab::Environments,
        CatalogueTab::Bestiary,
        CatalogueTab::Starters,
    ];

    fn label(&self) -> &'static str {
        match self {
            CatalogueTab::Attacks => "Attacks",
            CatalogueTab::Environments => "Environments",
            CatalogueTab::Bestiary => "Bestiary",
            CatalogueTab::Starters => "Starters",
        }
    }

    fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|t| t == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }
}

struct AttacksTab {
    attacks: Vec<Attack>,
    selected: usize,
    scroll: usize,
    last_list_height: u16,
}

impl AttacksTab {
    fn new() -> Self {
        Self {
            attacks: attack_lib::all_attacks(),
            selected: 0,
            scroll: 0,
            last_list_height: 0,
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        let len = self.attacks.len();
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.scroll {
                        self.scroll = self.selected;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < len {
                    self.selected += 1;
                    let visible = self.last_list_height as usize;
                    if visible > 0 && self.selected >= self.scroll + visible {
                        self.scroll = self.selected + 1 - visible;
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
            .split(area);
        let list_area = main_chunks[0];
        let right_area = main_chunks[1];

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(right_area);
        let visuals_area = right_chunks[0];
        let info_area = right_chunks[1];

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
    }
}

struct EnvironmentsTab {
    styles: &'static [GroundStyle],
    selected: usize,
    scroll: usize,
    time: TimeOfDay,
    last_list_height: u16,
    cached: Option<Environment>,
    cached_for: Option<(GroundStyle, TimeOfDay)>,
    cached_size: (u16, u16),
}

impl EnvironmentsTab {
    fn new() -> Self {
        Self {
            styles: GroundStyle::ALL,
            selected: 0,
            scroll: 0,
            time: TimeOfDay::Day,
            last_list_height: 0,
            cached: None,
            cached_for: None,
            cached_size: (0, 0),
        }
    }

    fn current_style(&self) -> Option<GroundStyle> {
        self.styles.get(self.selected).copied()
    }

    fn cycle_time(&mut self, delta: i32) {
        let times = TimeOfDay::ALL;
        let len = times.len() as i32;
        let idx = times.iter().position(|t| *t == self.time).unwrap_or(0) as i32;
        let next = ((idx + delta).rem_euclid(len)) as usize;
        self.time = times[next];
    }

    fn handle_key(&mut self, key: KeyCode) {
        let len = self.styles.len();
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.scroll {
                        self.scroll = self.selected;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < len {
                    self.selected += 1;
                    let visible = self.last_list_height as usize;
                    if visible > 0 && self.selected >= self.scroll + visible {
                        self.scroll = self.selected + 1 - visible;
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') => self.cycle_time(1),
            KeyCode::Left | KeyCode::Char('h') => self.cycle_time(-1),
            _ => {}
        }
    }

    fn update(&mut self) {
        if let Some(env) = self.cached.as_mut() {
            // Freeze time, drift clouds.
            env.update_cycle(0.05, 0.0, 1.0);
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
            .split(area);
        let visual_area = chunks[0];
        let list_area = chunks[1];

        self.refresh_cache(visual_area);
        render_env_visual(
            frame,
            self.cached.as_ref(),
            self.current_style(),
            self.time,
            visual_area,
        );

        self.last_list_height = list_area.height.saturating_sub(2);
        render_env_list(
            frame,
            self.styles,
            self.selected,
            self.scroll,
            list_area,
        );
    }

    fn refresh_cache(&mut self, visual_area: Rect) {
        let block_inner_size = (
            visual_area.width.saturating_sub(2),
            visual_area.height.saturating_sub(2),
        );
        if block_inner_size.0 == 0 || block_inner_size.1 == 0 {
            return;
        }
        let Some(style) = self.current_style() else {
            return;
        };
        let key = (style, self.time);
        let needs_rebuild = self.cached.is_none()
            || self.cached_for != Some(key)
            || self.cached_size != block_inner_size;
        if !needs_rebuild {
            return;
        }
        self.cached = Some(Environment::generate_at(
            block_inner_size.0,
            block_inner_size.1,
            style,
            self.time,
        ));
        self.cached_for = Some(key);
        self.cached_size = block_inner_size;
    }
}

fn render_env_visual(
    frame: &mut Frame,
    env: Option<&Environment>,
    style: Option<GroundStyle>,
    time: TimeOfDay,
    area: Rect,
) {
    let title = match style {
        Some(s) => format!(" {} — {} ", s.label(), time.label()),
        None => " Environment ".to_string(),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(env) = env else {
        return;
    };
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    widgets::render_environment_background(frame, env, inner);
    widgets::render_ground(frame, env, inner);
}

fn render_env_list(
    frame: &mut Frame,
    styles: &[GroundStyle],
    selected: usize,
    scroll: usize,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Environments ",
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
    let end = (scroll + visible).min(styles.len());

    let mut lines: Vec<Line> = Vec::with_capacity(end - scroll);
    for (offset, ground) in styles[scroll..end].iter().enumerate() {
        let idx = scroll + offset;
        let is_selected = idx == selected;
        let cursor = if is_selected { "▶ " } else { "  " };
        let row_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(vec![
            Span::styled(cursor, row_style),
            Span::styled(ground.label().to_string(), row_style),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

struct BestiaryTab {
    enemies: Vec<Enemy>,
    selected: usize,
    scroll: usize,
    last_list_height: u16,
}

impl BestiaryTab {
    fn new() -> Self {
        Self {
            enemies: enemy_lib::all_enemies(),
            selected: 0,
            scroll: 0,
            last_list_height: 0,
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        let len = self.enemies.len();
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.scroll {
                        self.scroll = self.selected;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < len {
                    self.selected += 1;
                    let visible = self.last_list_height as usize;
                    if visible > 0 && self.selected >= self.scroll + visible {
                        self.scroll = self.selected + 1 - visible;
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
            .split(area);
        let list_area = main_chunks[0];
        let right_area = main_chunks[1];

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(right_area);
        let visual_area = right_chunks[0];
        let info_area = right_chunks[1];

        self.last_list_height = list_area.height.saturating_sub(2);
        render_enemy_list(
            frame,
            &self.enemies,
            self.selected,
            self.scroll,
            list_area,
        );

        if let Some(enemy) = self.enemies.get(self.selected) {
            render_enemy_visual(frame, enemy, visual_area);
            render_enemy_info(frame, enemy, info_area);
        }
    }
}

fn render_enemy_list(
    frame: &mut Frame,
    enemies: &[Enemy],
    selected: usize,
    scroll: usize,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Bestiary ",
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
    let end = (scroll + visible).min(enemies.len());

    let mut lines: Vec<Line> = Vec::with_capacity(end - scroll);
    for (offset, enemy) in enemies[scroll..end].iter().enumerate() {
        let idx = scroll + offset;
        let is_selected = idx == selected;
        let cursor = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(enemy.primary_type.color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let prefix = if enemy.is_boss { "★ " } else { "" };
        lines.push(Line::from(vec![
            Span::styled(cursor, style),
            Span::styled(format!("{}{}", prefix, enemy.name), style),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_enemy_visual(frame: &mut Frame, enemy: &Enemy, area: Rect) {
    let title = if enemy.is_boss {
        format!(" ★ {} ", enemy.name)
    } else {
        format!(" {} ", enemy.name)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            title,
            Style::default()
                .fg(enemy.primary_type.color())
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // Type chips
    let mut type_spans = vec![Span::styled(
        enemy.primary_type.label().to_string(),
        Style::default()
            .fg(enemy.primary_type.color())
            .add_modifier(Modifier::BOLD),
    )];
    if let Some(secondary) = enemy.secondary_type {
        type_spans.push(Span::raw(" / "));
        type_spans.push(Span::styled(
            secondary.label().to_string(),
            Style::default()
                .fg(secondary.color())
                .add_modifier(Modifier::BOLD),
        ));
    }
    lines.push(Line::from(type_spans));
    lines.push(Line::from(""));

    // Sprite
    let sprite_height = enemy.sprite.len() as u16;
    let pad = inner.height.saturating_sub(sprite_height + 2) / 2;
    for _ in 0..pad {
        lines.push(Line::from(""));
    }
    for row in &enemy.sprite {
        lines.push(Line::from(Span::styled(
            row.clone(),
            Style::default().fg(enemy.color()),
        )));
    }

    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        inner,
    );
}

fn render_enemy_info(frame: &mut Frame, enemy: &Enemy, area: Rect) {
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

    // Stats
    lines.push(Line::from(vec![
        Span::styled("HP    ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", enemy.max_hp),
            Style::default().fg(Color::Rgb(255, 120, 120)),
        ),
        Span::raw("    "),
        Span::styled("Speed ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", enemy.speed),
            Style::default().fg(Color::Rgb(120, 200, 255)),
        ),
    ]));

    // Weaknesses
    let weaknesses = weaknesses_for(enemy);
    if !weaknesses.is_empty() {
        let mut spans = vec![Span::styled(
            "Weak  ",
            Style::default().fg(Color::DarkGray),
        )];
        for (idx, (el, mult)) in weaknesses.iter().enumerate() {
            if idx > 0 {
                spans.push(Span::raw(", "));
            }
            spans.push(Span::styled(
                format!("{} ({}x)", el.label(), format_mult(*mult)),
                Style::default()
                    .fg(el.color())
                    .add_modifier(Modifier::BOLD),
            ));
        }
        lines.push(Line::from(spans));
    }

    let resists = resistances_for(enemy);
    if !resists.is_empty() {
        let mut spans = vec![Span::styled(
            "Resist",
            Style::default().fg(Color::DarkGray),
        )];
        spans.push(Span::raw("  "));
        for (idx, (el, mult)) in resists.iter().enumerate() {
            if idx > 0 {
                spans.push(Span::raw(", "));
            }
            spans.push(Span::styled(
                format!("{} ({}x)", el.label(), format_mult(*mult)),
                Style::default().fg(el.color()),
            ));
        }
        lines.push(Line::from(spans));
    }

    // Moveset
    if !enemy.moveset.is_empty() {
        let moves = enemy.moveset.join(", ");
        lines.push(Line::from(vec![
            Span::styled("Moves ", Style::default().fg(Color::DarkGray)),
            Span::styled(moves, Style::default().fg(Color::Gray)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        enemy.description.clone(),
        Style::default().fg(Color::Gray),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn weaknesses_for(enemy: &Enemy) -> Vec<(Element, f32)> {
    type_effectiveness_summary(enemy)
        .into_iter()
        .filter(|(_, m)| *m > 1.0)
        .collect()
}

fn resistances_for(enemy: &Enemy) -> Vec<(Element, f32)> {
    type_effectiveness_summary(enemy)
        .into_iter()
        .filter(|(_, m)| *m < 1.0)
        .collect()
}

fn type_effectiveness_summary(enemy: &Enemy) -> Vec<(Element, f32)> {
    const ALL: &[Element] = &[
        Element::Normal,
        Element::Fire,
        Element::Water,
        Element::Grass,
        Element::Ice,
        Element::Electric,
        Element::Ground,
        Element::Flying,
        Element::Psychic,
    ];
    ALL.iter()
        .map(|el| {
            (
                *el,
                el.effectiveness_vs(enemy.primary_type, enemy.secondary_type),
            )
        })
        .collect()
}

fn format_mult(m: f32) -> String {
    if (m - m.round()).abs() < 0.01 {
        format!("{}", m.round() as i32)
    } else {
        format!("{}", m)
    }
}

const STARTER_FRAME_DURATION: f32 = 0.4;

struct StartersTab {
    starters: Vec<Starter>,
    selected: usize,
    scroll: usize,
    last_list_height: u16,
    crab: Crab,
    crab_bounds: (f32, f32),
    frame_timer: f32,
    frame_index: usize,
}

impl StartersTab {
    fn new() -> Self {
        let mut crab = Crab::new((0.0, 0.0), 95);
        crab.walk_range_x = Some((0.0, 0.5));
        Self {
            starters: starter_lib::all_starters(),
            selected: 0,
            scroll: 0,
            last_list_height: 0,
            crab,
            crab_bounds: (0.0, 0.0),
            frame_timer: 0.0,
            frame_index: 0,
        }
    }

    fn update(&mut self) {
        if self.crab_bounds.0 > 0.0 && self.crab_bounds.1 > 0.0 {
            self.crab.update(0.05, self.crab_bounds);
        }
        self.frame_timer += 0.05;
        if self.frame_timer >= STARTER_FRAME_DURATION {
            self.frame_timer -= STARTER_FRAME_DURATION;
            self.frame_index = self.frame_index.wrapping_add(1);
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        let len = self.starters.len();
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.scroll {
                        self.scroll = self.selected;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < len {
                    self.selected += 1;
                    let visible = self.last_list_height as usize;
                    if visible > 0 && self.selected >= self.scroll + visible {
                        self.scroll = self.selected + 1 - visible;
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
            .split(area);
        let list_area = main_chunks[0];
        let right_area = main_chunks[1];

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(right_area);
        let visual_area = right_chunks[0];
        let info_area = right_chunks[1];

        self.last_list_height = list_area.height.saturating_sub(2);
        render_starter_list(
            frame,
            &self.starters,
            self.selected,
            self.scroll,
            list_area,
        );

        self.update_crab_bounds(visual_area);
        if let Some(starter) = self.starters.get(self.selected) {
            render_starter_visual(
                frame,
                starter,
                &self.crab,
                self.frame_index,
                visual_area,
            );
            render_starter_info(frame, starter, info_area);
        }
    }

    /// Re-center the crab inside the visual panel when the area resizes.
    /// `crab_bounds` drives ground/jump math in Crab::update; with a 1-wide
    /// `walk_range_x` the crab can't drift horizontally.
    fn update_crab_bounds(&mut self, visual_area: Rect) {
        let block_inner = (
            visual_area.width.saturating_sub(2),
            visual_area.height.saturating_sub(2),
        );
        // The visual panel splits its inner into a 2-line type chip + sprite area.
        let sprite_h = block_inner.1.saturating_sub(2);
        let bounds = (block_inner.0 as f32, sprite_h as f32);
        if bounds == self.crab_bounds || bounds.0 <= 0.0 || bounds.1 <= 0.0 {
            return;
        }
        let cx = ((bounds.0 - 15.0).max(0.0)) / 2.0;
        self.crab.position.0 = cx;
        self.crab.walk_range_x = Some((cx, cx + 0.5));
        self.crab_bounds = bounds;
    }
}

fn render_starter_list(
    frame: &mut Frame,
    starters: &[Starter],
    selected: usize,
    scroll: usize,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Starters ",
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
    let end = (scroll + visible).min(starters.len());

    let mut lines: Vec<Line> = Vec::with_capacity(end - scroll);
    for (offset, starter) in starters[scroll..end].iter().enumerate() {
        let idx = scroll + offset;
        let is_selected = idx == selected;
        let cursor = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default()
                .fg(starter.primary_type.color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(vec![
            Span::styled(cursor, style),
            Span::styled(starter.name.clone(), style),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_starter_visual(
    frame: &mut Frame,
    starter: &Starter,
    crab: &Crab,
    frame_index: usize,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            format!(" {} ", starter.name),
            Style::default()
                .fg(starter.primary_type.color())
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(inner);
    let chip_area = chunks[0];
    let sprite_area = chunks[1];

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
            crate::ui::widgets::render_crab(frame, crab, sprite_area, None);
        }
        StarterVisual::Frames(frames) => {
            if frames.is_empty() {
                return;
            }
            let sprite = &frames[frame_index % frames.len()];
            let sprite_height = sprite.len() as u16;
            let mut lines: Vec<Line> = Vec::with_capacity(sprite.len() + 4);
            let pad = sprite_area
                .height
                .saturating_sub(sprite_height)
                / 2;
            for _ in 0..pad {
                lines.push(Line::from(""));
            }
            for row in sprite {
                lines.push(Line::from(Span::styled(
                    row.clone(),
                    Style::default()
                        .fg(starter.color())
                        .add_modifier(Modifier::BOLD),
                )));
            }
            frame.render_widget(
                Paragraph::new(lines).alignment(Alignment::Center),
                sprite_area,
            );
        }
    }
}

fn render_starter_info(frame: &mut Frame, starter: &Starter, area: Rect) {
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
    lines.push(Line::from(vec![
        Span::styled("Type   ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            starter.primary_type.label().to_string(),
            Style::default()
                .fg(starter.primary_type.color())
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    if !starter.starting_attacks.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Moves  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                starter.starting_attacks.join(", "),
                Style::default().fg(Color::Gray),
            ),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        starter.description.clone(),
        Style::default().fg(Color::Gray),
    )));
    frame.render_widget(Paragraph::new(lines), inner);
}

pub struct CatalogueScreen {
    tab: CatalogueTab,
    attacks: AttacksTab,
    environments: EnvironmentsTab,
    bestiary: BestiaryTab,
    starters: StartersTab,
}

impl CatalogueScreen {
    pub fn new() -> Self {
        Self {
            tab: CatalogueTab::Attacks,
            attacks: AttacksTab::new(),
            environments: EnvironmentsTab::new(),
            bestiary: BestiaryTab::new(),
            starters: StartersTab::new(),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Transition::Goto(Screen::Select(SelectScreen::new()));
            }
            KeyCode::Tab => {
                self.tab = self.tab.next();
                return Transition::Stay;
            }
            _ => {}
        }
        match self.tab {
            CatalogueTab::Attacks => self.attacks.handle_key(key),
            CatalogueTab::Environments => self.environments.handle_key(key),
            CatalogueTab::Bestiary => self.bestiary.handle_key(key),
            CatalogueTab::Starters => self.starters.handle_key(key),
        }
        Transition::Stay
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        match self.tab {
            CatalogueTab::Environments => self.environments.update(),
            CatalogueTab::Starters => self.starters.update(),
            _ => {}
        }
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
        let area = frame.area();
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(1),
            ])
            .split(area);
        let header_area = v_chunks[0];
        let tab_area = v_chunks[1];
        let body_area = v_chunks[2];
        let hint_area = v_chunks[3];

        render_header(frame, header_area);
        render_tabs(frame, self.tab, tab_area);

        match self.tab {
            CatalogueTab::Attacks => self.attacks.draw(frame, body_area),
            CatalogueTab::Environments => self.environments.draw(frame, body_area),
            CatalogueTab::Bestiary => self.bestiary.draw(frame, body_area),
            CatalogueTab::Starters => self.starters.draw(frame, body_area),
        }

        render_hint(frame, hint_area);
    }
}

fn render_tabs(frame: &mut Frame, current: CatalogueTab, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();
    for (idx, tab) in CatalogueTab::ALL.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled("   ", Style::default().fg(Color::DarkGray)));
        }
        let is_active = *tab == current;
        let style = if is_active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let label = if is_active {
            format!("[ {} ]", tab.label())
        } else {
            format!("  {}  ", tab.label())
        };
        spans.push(Span::styled(label, style));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
        area,
    );
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
        Span::styled("Tab", key),
        Span::styled(" switch tab   ", dim),
        Span::styled("↑ ↓", key),
        Span::styled(" scroll   ", dim),
        Span::styled("← →", key),
        Span::styled(" time of day   ", dim),
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
