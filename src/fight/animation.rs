use super::attack::AnimationKind;
use super::projectile::ProjectileKind;

const JUMP_DURATION: f32 = 0.8;
const DASH_DURATION: f32 = 0.5;
const THROW_DURATION: f32 = 0.6;
const JUMP_HEIGHT: f32 = 4.0;

#[derive(Debug, Clone)]
pub struct Animation {
    pub kind: AnimationKind,
    pub start_x: f32,
    pub target_x: f32,
    pub elapsed: f32,
    pub duration: f32,
}

impl Animation {
    pub fn new(kind: AnimationKind, start_x: f32, target_x: f32) -> Self {
        let duration = match kind {
            AnimationKind::Jump => JUMP_DURATION,
            AnimationKind::Dash => DASH_DURATION,
            AnimationKind::Throw(_) => THROW_DURATION,
        };
        Self {
            kind,
            start_x,
            target_x,
            elapsed: 0.0,
            duration,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    pub fn is_done(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    pub fn crab_position(&self, base: (f32, f32)) -> (f32, f32) {
        let p = self.progress();
        match self.kind {
            AnimationKind::Jump => {
                let (lerp, phase) = if p < 0.5 {
                    let phase = p * 2.0;
                    (phase, phase)
                } else {
                    let phase = (p - 0.5) * 2.0;
                    (1.0 - phase, phase)
                };
                let x = self.start_x + (self.target_x - self.start_x) * lerp;
                let y_offset = -JUMP_HEIGHT * (std::f32::consts::PI * phase).sin();
                (x, base.1 + y_offset)
            }
            AnimationKind::Dash => {
                let lerp = if p < 0.5 { p * 2.0 } else { (1.0 - p) * 2.0 };
                let x = self.start_x + (self.target_x - self.start_x) * lerp;
                (x, base.1)
            }
            AnimationKind::Throw(_) => (self.start_x, base.1),
        }
    }

    pub fn projectile_position(&self, base_y: f32) -> Option<(f32, f32)> {
        match self.kind {
            AnimationKind::Throw(_) => {
                let p = self.progress();
                let x = self.start_x + 12.0 + (self.target_x - self.start_x - 12.0) * p;
                Some((x, base_y - 1.0))
            }
            _ => None,
        }
    }

    pub fn projectile_kind(&self) -> Option<ProjectileKind> {
        match self.kind {
            AnimationKind::Throw(p) => Some(p),
            _ => None,
        }
    }
}
