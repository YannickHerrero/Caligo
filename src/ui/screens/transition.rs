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
enum WipeShape {
    Horizontal,
    Vertical,
    Radial,
}

#[derive(Debug, Clone, Copy)]
enum Decoration {
    None,
    Glyph(char),
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
    fn color(&self) -> Color {
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

    fn label(&self) -> &'static str {
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

    fn shape(&self) -> WipeShape {
        match self {
            TransitionKind::EasyFight | TransitionKind::NormalFight => WipeShape::Horizontal,
            TransitionKind::EliteFight | TransitionKind::Boss => WipeShape::Horizontal,
            TransitionKind::Camp | TransitionKind::Mystery => WipeShape::Radial,
            TransitionKind::Shop => WipeShape::Vertical,
        }
    }

    fn decoration(&self) -> Decoration {
        match self {
            TransitionKind::EliteFight => Decoration::Glyph('⚜'),
            TransitionKind::Camp => Decoration::Glyph('✦'),
            TransitionKind::Shop => Decoration::Glyph('◆'),
            TransitionKind::Mystery => Decoration::Glyph('?'),
            TransitionKind::Boss => Decoration::Glyph('☠'),
            _ => Decoration::None,
        }
    }
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
        match self.kind.shape() {
            WipeShape::Horizontal => self.draw_horizontal(frame, intensity),
            WipeShape::Vertical => self.draw_vertical(frame, intensity),
            WipeShape::Radial => self.draw_radial(frame, intensity),
        }
    }

    fn draw_horizontal(&self, frame: &mut Frame, intensity: f32) {
        let area = frame.area();
        let color = self.kind.color();
        let bar_h = ((area.height as f32 * intensity / 2.0).round() as u16).min(area.height / 2 + 1);
        if bar_h == 0 {
            return;
        }
        let buf = frame.buffer_mut();
        for y in area.y..(area.y + bar_h).min(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                paint_cell(buf, x, y, color, self.decoration_for(x, y));
            }
        }
        let bot_start = (area.y + area.height).saturating_sub(bar_h);
        for y in bot_start..(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                paint_cell(buf, x, y, color, self.decoration_for(x, y));
            }
        }
    }

    fn draw_vertical(&self, frame: &mut Frame, intensity: f32) {
        let area = frame.area();
        let color = self.kind.color();
        let bar_w = ((area.width as f32 * intensity / 2.0).round() as u16).min(area.width / 2 + 1);
        if bar_w == 0 {
            return;
        }
        let buf = frame.buffer_mut();
        for x in area.x..(area.x + bar_w).min(area.x + area.width) {
            for y in area.y..(area.y + area.height) {
                paint_cell(buf, x, y, color, self.decoration_for(x, y));
            }
        }
        let right_start = (area.x + area.width).saturating_sub(bar_w);
        for x in right_start..(area.x + area.width) {
            for y in area.y..(area.y + area.height) {
                paint_cell(buf, x, y, color, self.decoration_for(x, y));
            }
        }
    }

    fn draw_radial(&self, frame: &mut Frame, intensity: f32) {
        // Fill cells whose normalized distance from center exceeds (1 - intensity).
        let area = frame.area();
        let color = self.kind.color();
        if intensity <= 0.0 {
            return;
        }
        let cx = area.x as f32 + area.width as f32 / 2.0;
        let cy = area.y as f32 + area.height as f32 / 2.0;
        // Use a 2:1 cell aspect ratio so the "circle" looks round on terminal cells.
        let nx = (area.width as f32 / 2.0).max(1.0);
        let ny = (area.height as f32 / 2.0).max(1.0);
        let threshold = (1.0 - intensity).clamp(0.0, 1.0);
        let buf = frame.buffer_mut();
        for y in area.y..(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                let dx = (x as f32 - cx) / nx;
                let dy = (y as f32 - cy) / ny;
                let d = (dx * dx + dy * dy).sqrt();
                if d >= threshold {
                    paint_cell(buf, x, y, color, self.decoration_for(x, y));
                }
            }
        }
    }

    fn decoration_for(&self, x: u16, y: u16) -> Option<char> {
        match self.kind.decoration() {
            Decoration::None => None,
            Decoration::Glyph(g) => {
                // Sparse stippling — about 1 in 9 cells gets a glyph, varied
                // by tick so the pattern crawls a bit during the transition.
                let h = (x as u32).wrapping_mul(73)
                    ^ (y as u32).wrapping_mul(151)
                    ^ (self.tick / 2).wrapping_mul(31);
                if h % 9 == 0 {
                    Some(g)
                } else {
                    None
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

fn paint_cell(buf: &mut ratatui::buffer::Buffer, x: u16, y: u16, color: Color, glyph: Option<char>) {
    let Some(cell) = buf.cell_mut((x, y)) else {
        return;
    };
    match glyph {
        None => {
            cell.set_char(' ').set_bg(color);
        }
        Some(g) => {
            cell.set_char(g)
                .set_fg(Color::Black)
                .set_bg(color)
                .set_style(Style::default().add_modifier(Modifier::BOLD).bg(color).fg(Color::Black));
        }
    }
}
