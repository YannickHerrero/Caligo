use crate::map::NodeKind;
use crate::ui::screen::{Screen, Transition};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[derive(Debug, Clone, Copy)]
pub enum TransitionKind {
    EasyFight,
    NormalFight,
    EliteFight,
    Camp,
    Shop,
    Mystery,
    Boss,
}

#[derive(Debug, Clone, Copy)]
enum TransitionEffect {
    HorizontalBars,
    VerticalCurtains,
    IrisClose,
    DiagonalSlash,
    RandomScatter,
    SpiralInward,
    Checkerboard,
    DiamondExpand,
}

impl From<NodeKind> for TransitionKind {
    fn from(k: NodeKind) -> Self {
        match k {
            NodeKind::EasyFight => TransitionKind::EasyFight,
            NodeKind::NormalFight => TransitionKind::NormalFight,
            NodeKind::EliteFight => TransitionKind::EliteFight,
            NodeKind::Camp => TransitionKind::Camp,
            NodeKind::Shop => TransitionKind::Shop,
            NodeKind::Mystery => TransitionKind::Mystery,
            NodeKind::Boss => TransitionKind::Boss,
        }
    }
}

impl TransitionKind {
    pub const ALL: &'static [TransitionKind] = &[
        TransitionKind::EasyFight,
        TransitionKind::NormalFight,
        TransitionKind::EliteFight,
        TransitionKind::Camp,
        TransitionKind::Shop,
        TransitionKind::Mystery,
        TransitionKind::Boss,
    ];

    pub fn color(&self) -> Color {
        match self {
            TransitionKind::EasyFight => Color::Rgb(120, 200, 120),
            TransitionKind::NormalFight => Color::Rgb(220, 90, 90),
            TransitionKind::EliteFight => Color::Rgb(255, 150, 60),
            TransitionKind::Camp => Color::Rgb(255, 210, 110),
            TransitionKind::Shop => Color::Rgb(110, 210, 230),
            TransitionKind::Mystery => Color::Rgb(190, 130, 230),
            TransitionKind::Boss => Color::Rgb(230, 70, 130),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TransitionKind::EasyFight => "Fight!",
            TransitionKind::NormalFight => "Fight!",
            TransitionKind::EliteFight => "Elite!",
            TransitionKind::Camp => "Rest",
            TransitionKind::Shop => "Shop",
            TransitionKind::Mystery => "?",
            TransitionKind::Boss => "BOSS",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TransitionKind::EasyFight => "Easy Fight",
            TransitionKind::NormalFight => "Normal Fight",
            TransitionKind::EliteFight => "Elite Fight",
            TransitionKind::Camp => "Campment",
            TransitionKind::Shop => "Shop",
            TransitionKind::Mystery => "Mystery",
            TransitionKind::Boss => "Boss",
        }
    }

    fn duration(&self) -> u32 {
        match self {
            TransitionKind::EasyFight => 16,
            TransitionKind::NormalFight => 22,
            TransitionKind::EliteFight => 28,
            TransitionKind::Camp => 30,
            TransitionKind::Shop => 22,
            TransitionKind::Mystery => 26,
            TransitionKind::Boss => 38,
        }
    }

    fn effect(&self) -> TransitionEffect {
        match self {
            TransitionKind::EasyFight => TransitionEffect::DiagonalSlash,
            TransitionKind::NormalFight => TransitionEffect::RandomScatter,
            TransitionKind::EliteFight => TransitionEffect::SpiralInward,
            TransitionKind::Camp => TransitionEffect::IrisClose,
            TransitionKind::Shop => TransitionEffect::Checkerboard,
            TransitionKind::Mystery => TransitionEffect::DiamondExpand,
            TransitionKind::Boss => TransitionEffect::HorizontalBars,
        }
    }
}

impl TransitionEffect {
    fn covers(&self, x: u16, y: u16, area: Rect, intensity: f32) -> bool {
        match self {
            TransitionEffect::HorizontalBars => {
                let bar_h = area.height as f32 * intensity / 2.0;
                let dy = (y as i32 - area.y as i32) as f32;
                let from_top = dy;
                let from_bot = (area.height as f32 - 1.0) - dy;
                from_top < bar_h || from_bot < bar_h
            }
            TransitionEffect::VerticalCurtains => {
                let bar_w = area.width as f32 * intensity / 2.0;
                let dx = (x as i32 - area.x as i32) as f32;
                let from_left = dx;
                let from_right = (area.width as f32 - 1.0) - dx;
                from_left < bar_w || from_right < bar_w
            }
            TransitionEffect::IrisClose => {
                let (dx, dy) = aspect_distance(x, y, area);
                let d = (dx * dx + dy * dy).sqrt();
                d >= (1.0 - intensity)
            }
            TransitionEffect::DiagonalSlash => {
                // Sweep diagonally from top-left to bottom-right. The thin
                // tilt around the threshold gives the band a clean slash
                // feel rather than a fade.
                let w = area.width.max(1) as f32;
                let h = area.height.max(1) as f32;
                let dx = (x as f32 - area.x as f32) / w;
                let dy = (y as f32 - area.y as f32) / h;
                let progress = (dx + dy) * 0.5;
                progress < intensity
            }
            TransitionEffect::RandomScatter => {
                // Pokemon trainer-encounter feel: 2-wide blocks fill in a
                // pseudo-random order driven by a per-block hash. Each block
                // gets a stable threshold so the same cells always fill at
                // the same point in the animation.
                let bx = ((x as i32 - area.x as i32) / 2) as u32;
                let by = (y as i32 - area.y as i32) as u32;
                let threshold = scatter_threshold(bx, by);
                threshold < intensity
            }
            TransitionEffect::SpiralInward => {
                // Outer cells with angle near 0 fill first; the front then
                // sweeps clockwise and steadily inward, sketching a spiral
                // arm. arms=3 gives several visible turns before the center
                // is reached.
                let (dx, dy) = aspect_distance(x, y, area);
                let d = (dx * dx + dy * dy).sqrt().min(1.5);
                let theta = dy.atan2(dx);
                let theta_norm = theta / std::f32::consts::TAU + 0.5;
                let arms = 3.0;
                let max_fill = 1.5 + arms;
                let fill_at = (1.5 - d) + arms * theta_norm;
                intensity * max_fill > fill_at
            }
            TransitionEffect::DiamondExpand => {
                // Diamond (Manhattan distance) grows from center outward.
                // Manhattan max is 2.0 in aspect-corrected coords (corner);
                // we cap so corners get covered at intensity 1.
                let (dx, dy) = aspect_distance(x, y, area);
                let manhattan = (dx.abs() + dy.abs()).min(2.0);
                intensity * 2.0 >= manhattan
            }
            TransitionEffect::Checkerboard => {
                // Two-pass tile fill in 2x1 squares (the cell aspect makes
                // 2 wide x 1 tall look square). Phase 1 (intensity 0 -> 0.5)
                // fills the even-parity squares left-to-right, top-to-bottom;
                // phase 2 (0.5 -> 1.0) fills the odd-parity squares the same
                // way over what's already painted.
                let bx = ((x as i32 - area.x as i32) / 2).max(0);
                let by = (y as i32 - area.y as i32).max(0);
                let parity = (bx + by).rem_euclid(2);
                let cells_per_row = (area.width as i32 / 2).max(1);
                let total = (cells_per_row as f32 * area.height as f32).max(1.0);
                let block_idx = (by * cells_per_row + bx) as f32;
                let p_norm = block_idx / total;
                if parity == 0 {
                    intensity * 2.0 > p_norm
                } else {
                    (intensity - 0.5) * 2.0 > p_norm
                }
            }
        }
    }
}

fn scatter_threshold(bx: u32, by: u32) -> f32 {
    let mut h = bx.wrapping_mul(73_856_093) ^ by.wrapping_mul(19_349_663);
    h = h.wrapping_mul(2_654_435_761);
    h ^= h >> 16;
    (h % 1024) as f32 / 1024.0
}

fn aspect_distance(x: u16, y: u16, area: Rect) -> (f32, f32) {
    let cx = area.x as f32 + area.width as f32 / 2.0;
    let cy = area.y as f32 + area.height as f32 / 2.0;
    let nx = (area.width as f32 / 2.0).max(1.0);
    let ny = (area.height as f32 / 2.0).max(1.0);
    ((x as f32 - cx) / nx, (y as f32 - cy) / ny)
}

pub struct TransitionScreen {
    pub from: Box<Screen>,
    pub to: Option<Box<Screen>>,
    pub kind: TransitionKind,
    pub tick: u32,
    pub duration: u32,
}

impl TransitionScreen {
    pub fn new(from: Screen, to: Screen, kind: TransitionKind) -> Self {
        let duration = kind.duration();
        Self {
            from: Box::new(from),
            to: Some(Box::new(to)),
            kind,
            tick: 0,
            duration,
        }
    }

    fn progress(&self) -> f32 {
        (self.tick as f32 / self.duration as f32).clamp(0.0, 1.0)
    }

    pub fn handle_key(&mut self, _key: KeyCode) -> Transition {
        Transition::Stay
    }

    pub fn update(&mut self) -> Transition {
        self.tick = self.tick.saturating_add(1);
        if self.tick >= self.duration {
            if let Some(to) = self.to.take() {
                return Transition::Goto(*to);
            }
        }
        Transition::Stay
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let p = self.progress();
        if p < 0.5 {
            self.from.draw(frame);
            self.draw_wipe(frame, p * 2.0);
        } else {
            if let Some(to) = self.to.as_mut() {
                to.draw(frame);
            }
            self.draw_wipe(frame, 2.0 - p * 2.0);
        }
        if p >= 0.35 && p <= 0.65 {
            self.draw_label(frame);
        }
    }

    fn draw_wipe(&self, frame: &mut Frame, intensity: f32) {
        if intensity <= 0.0 {
            return;
        }
        let area = frame.area();
        let color = self.kind.color();
        let effect = self.kind.effect();
        let buf = frame.buffer_mut();
        for y in area.y..(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                if effect.covers(x, y, area, intensity) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(' ').set_bg(color);
                    }
                }
            }
        }
    }

    fn draw_label(&self, frame: &mut Frame) {
        let area = frame.area();
        if area.height < 3 || area.width < 8 {
            return;
        }
        let label = self.kind.label();
        let text_w = (label.chars().count() as u16 + 4).min(area.width);
        let row = area.y + area.height / 2;
        let col = area.x + (area.width.saturating_sub(text_w)) / 2;
        let label_area = Rect {
            x: col,
            y: row,
            width: text_w,
            height: 1,
        };
        let line = Line::from(Span::styled(
            format!("  {}  ", label),
            Style::default()
                .fg(Color::Black)
                .bg(self.kind.color())
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(Paragraph::new(line), label_area);
    }
}
