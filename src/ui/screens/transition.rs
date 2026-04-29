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

const DEFAULT_DURATION: u32 = 24;

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
        Self {
            from: Box::new(from),
            to: Some(Box::new(to)),
            kind,
            tick: 0,
            duration: DEFAULT_DURATION,
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
        let area = frame.area();
        let color = self.kind.color();
        let bar_h = ((area.height as f32 * intensity / 2.0).round() as u16).min(area.height / 2 + 1);
        if bar_h == 0 {
            return;
        }
        let buf = frame.buffer_mut();
        // Top bar
        for y in area.y..(area.y + bar_h).min(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ').set_bg(color);
                }
            }
        }
        // Bottom bar
        let bot_start = (area.y + area.height).saturating_sub(bar_h);
        for y in bot_start..(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ').set_bg(color);
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
