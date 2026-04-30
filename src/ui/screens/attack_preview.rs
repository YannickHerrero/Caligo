use crate::crab::Crab;
use crate::data::attacks as attack_lib;
use crate::environment::{Environment, GroundStyle};
use crate::fight::{Animation, Attack, Enemy};
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

pub struct AttackPreviewScreen {
    pub crab: Crab,
    pub environment: Environment,
    pub target: Enemy,
    pub attacks: Vec<Attack>,
    pub selected: usize,
    pub scroll: usize,
    pub animation: Option<Animation>,
    last_terminal_size: (u16, u16),
    last_list_height: u16,
}

impl AttackPreviewScreen {
    pub fn new() -> Self {
        Self {
            crab: Crab::new((6.0, 100.0), 95),
            environment: Environment::generate(80, 15, GroundStyle::default()),
            target: Enemy::slime(),
            attacks: attack_lib::all_attacks(),
            selected: 0,
            scroll: 0,
            animation: None,
            last_terminal_size: (0, 0),
            last_list_height: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, _player: &mut Player) -> Transition {
        if self.animation.is_some() {
            return Transition::Stay;
        }
        let len = self.attacks.len();
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Transition::Goto(Screen::Select(SelectScreen::new()));
            }
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
            KeyCode::Enter => {
                self.start_animation();
            }
            _ => {}
        }
        Transition::Stay
    }

    fn start_animation(&mut self) {
        if self.selected >= self.attacks.len() {
            return;
        }
        let kind = self.attacks[self.selected].kind;
        let start_x = self.crab.position.0;
        let target_x = (self.last_terminal_size.0 as f32 - 18.0).max(start_x + 5.0);
        self.animation = Some(Animation::new(kind, start_x, target_x));
    }

    pub fn update(&mut self, _player: &mut Player) -> Transition {
        let dt = 0.05;
        let bounds = (
            self.last_terminal_size.0 as f32 - 2.0,
            self.last_terminal_size.1 as f32,
        );

        if let Some(anim) = self.animation.as_mut() {
            anim.tick(dt);
            if anim.is_done() {
                self.animation = None;
            }
        } else if bounds.0 > 0.0 && bounds.1 > 0.0 {
            self.crab.walk_range_x = Some((0.0, bounds.0 * 0.4));
            self.crab.update(dt, bounds);
        }

        self.environment.update_cycle(dt, 1.0, 1.0);
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame, _player: &Player) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Length(1),
            ])
            .split(area);

        let header_area = chunks[0];
        let scene_area = chunks[1];
        let info_area = chunks[2];
        let list_area = chunks[3];
        let hint_area = chunks[4];

        let current_size = (scene_area.width, scene_area.height);
        if current_size != self.last_terminal_size {
            self.environment = Environment::generate(
                scene_area.width,
                scene_area.height,
                self.environment.ground_style,
            );
            self.last_terminal_size = current_size;
        }

        render_header(frame, header_area);

        let crab_override = self
            .animation
            .as_ref()
            .map(|anim| anim.crab_position(self.crab.position));

        widgets::render_environment_background(frame, &self.environment, scene_area);
        widgets::render_crab(frame, &self.crab, scene_area, crab_override);
        widgets::render_enemy(frame, &self.target, scene_area);
        if let Some(anim) = self.animation.as_ref() {
            widgets::render_projectile(frame, anim, self.crab.position.1, scene_area);
            widgets::render_particles(frame, anim, self.crab.position.1, scene_area);
        }
        widgets::render_ground(frame, &self.environment, scene_area);

        if let Some(attack) = self.attacks.get(self.selected) {
            render_info(frame, attack, info_area);
        }

        self.last_list_height = list_area.height.saturating_sub(2);
        render_list(
            frame,
            &self.attacks,
            self.selected,
            self.scroll,
            list_area,
        );
        render_hint(frame, hint_area);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            "Caligo — Attack Preview",
            Style::default()
                .fg(Color::Rgb(255, 140, 90))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   ·   ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "press Enter to play the selected attack",
            Style::default().fg(Color::Gray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn render_info(frame: &mut Frame, attack: &Attack, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let stats = Line::from(vec![
        Span::styled(
            attack.name.clone(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(attack.effect.label(), Style::default().fg(attack.effect.color())),
        Span::raw("  "),
        Span::styled(
            format!("MP {}", attack.mana_cost),
            Style::default().fg(Color::Rgb(120, 160, 255)),
        ),
        Span::raw("  "),
        Span::styled(
            attack.element.label().to_string(),
            Style::default().fg(attack.element.color()),
        ),
        Span::raw("  "),
        Span::styled(
            animation_label(attack),
            Style::default().fg(Color::Rgb(180, 180, 180)),
        ),
    ]);
    let desc = Line::from(Span::styled(
        attack.description.clone(),
        Style::default().fg(Color::Gray),
    ));
    frame.render_widget(Paragraph::new(vec![stats, desc]), inner);
}

fn animation_label(attack: &Attack) -> String {
    use crate::fight::AnimationKind;
    match attack.kind {
        AnimationKind::Jump => "Jump".to_string(),
        AnimationKind::Dash => "Dash".to_string(),
        AnimationKind::Throw(p) => format!("Throw({:?})", p),
        AnimationKind::SelfCast(p) => format!("SelfCast({:?})", p),
    }
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
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(vec![
            Span::styled(cursor, name_style),
            Span::styled(format!("{:<16}", attack.name), name_style),
            Span::styled(
                format!("{:<14}", attack.effect.label()),
                Style::default().fg(attack.effect.color()),
            ),
            Span::styled(
                format!("MP {:<3}", attack.mana_cost),
                Style::default().fg(Color::Rgb(120, 160, 255)),
            ),
            Span::styled(
                attack.element.label().to_string(),
                Style::default().fg(attack.element.color()),
            ),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_hint(frame: &mut Frame, area: Rect) {
    let key = Style::default().fg(Color::Yellow);
    let dim = Style::default().fg(Color::DarkGray);
    let hint = Line::from(vec![
        Span::styled("↑ ↓", key),
        Span::styled(" scroll   ", dim),
        Span::styled("Enter", key),
        Span::styled(" play   ", dim),
        Span::styled("q", key),
        Span::styled(" back", dim),
    ]);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), area);
}
