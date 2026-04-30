use super::attack::{AnimationKind, Attack, Effect, Element};
use super::particle::ParticleKind;
use super::projectile::{ProjectileKind, ProjectileSize};

const JUMP_DURATION: f32 = 0.8;
const DASH_DURATION: f32 = 0.5;
const THROW_DURATION: f32 = 0.6;
const SELF_CAST_DURATION: f32 = 0.9;
const JUMP_HEIGHT: f32 = 4.0;
const SELF_CAST_HEIGHT: f32 = 2.0;
const THROW_ARC_HEIGHT: f32 = 5.0;
const PARTICLE_COUNT: usize = 10;
const PARTICLE_LIFE: f32 = 0.5;
const CRAB_HALF_WIDTH: f32 = 11.0;
const CRAB_HALF_HEIGHT: f32 = 1.5;
const TRAIL_SAMPLES: usize = 6;
const TRAIL_STEP: f32 = 0.06;
const IMPACT_DURATION: f32 = 1.0;
const IMPACT_PARTICLE_COUNT: usize = 8;
const IMPACT_PARTICLE_LIFE: f32 = 0.45;

#[derive(Debug, Clone)]
pub struct Animation {
    pub kind: AnimationKind,
    pub projectile_size: ProjectileSize,
    pub trail: Option<ParticleKind>,
    pub impact: Option<ParticleKind>,
    pub start_x: f32,
    pub target_x: f32,
    pub elapsed: f32,
    pub move_duration: f32,
    pub total_duration: f32,
}

impl Animation {
    pub fn new(kind: AnimationKind, start_x: f32, target_x: f32) -> Self {
        Self::build(kind, ProjectileSize::Small, None, None, start_x, target_x)
    }

    pub fn for_attack(attack: &Attack, start_x: f32, target_x: f32) -> Self {
        let size = match attack.effect {
            Effect::Damage(d) => ProjectileSize::for_damage(d),
            _ => ProjectileSize::Small,
        };
        let trail = trail_for(attack.kind, attack.element);
        let impact = impact_for(attack.kind, attack.element, &attack.effect);
        Self::build(attack.kind, size, trail, impact, start_x, target_x)
    }

    fn build(
        kind: AnimationKind,
        projectile_size: ProjectileSize,
        trail: Option<ParticleKind>,
        impact: Option<ParticleKind>,
        start_x: f32,
        target_x: f32,
    ) -> Self {
        let move_duration = match kind {
            AnimationKind::Jump => JUMP_DURATION,
            AnimationKind::Dash => DASH_DURATION,
            AnimationKind::Throw(_) => THROW_DURATION,
            AnimationKind::SelfCast(_) => SELF_CAST_DURATION,
        };
        let mut total_duration = move_duration;
        if impact.is_some() {
            if let Some(start) = impact_start_t(kind, move_duration) {
                total_duration = total_duration.max(start + IMPACT_DURATION);
            }
        }
        Self {
            kind,
            projectile_size,
            trail,
            impact,
            start_x,
            target_x,
            elapsed: 0.0,
            move_duration,
            total_duration,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    pub fn is_done(&self) -> bool {
        self.elapsed >= self.total_duration
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.move_duration).clamp(0.0, 1.0)
    }

    pub fn crab_position(&self, base: (f32, f32)) -> (f32, f32) {
        self.crab_position_at(self.progress(), base)
    }

    fn crab_position_at(&self, p: f32, base: (f32, f32)) -> (f32, f32) {
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
            AnimationKind::SelfCast(_) => {
                let y_offset = -SELF_CAST_HEIGHT * (std::f32::consts::PI * p).sin();
                (self.start_x, base.1 + y_offset)
            }
        }
    }

    pub fn projectile_position(&self, base_y: f32) -> Option<(f32, f32)> {
        match self.kind {
            AnimationKind::Throw(_) => {
                if self.elapsed >= self.move_duration {
                    return None;
                }
                let p = self.progress();
                let x = self.start_x + 12.0 + (self.target_x - self.start_x - 12.0) * p;
                let arc = THROW_ARC_HEIGHT * (std::f32::consts::PI * p).sin();
                Some((x, base_y + 2.0 - arc))
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

    pub fn particles(&self, base: (f32, f32)) -> Vec<Particle> {
        let mut out = Vec::new();
        match self.kind {
            AnimationKind::SelfCast(_) => out.extend(self.aura_particles(base)),
            AnimationKind::Jump | AnimationKind::Dash => out.extend(self.trail_particles(base)),
            _ => {}
        }
        out.extend(self.impact_particles(base));
        out
    }

    fn aura_particles(&self, base: (f32, f32)) -> Vec<Particle> {
        let kind = match self.kind {
            AnimationKind::SelfCast(k) => k,
            _ => return Vec::new(),
        };
        let center_x = self.start_x + CRAB_HALF_WIDTH;
        let center_y = base.1 + CRAB_HALF_HEIGHT;
        let stagger_window = (self.move_duration - PARTICLE_LIFE).max(0.0);
        let mut out = Vec::with_capacity(PARTICLE_COUNT);
        for i in 0..PARTICLE_COUNT {
            let spawn_t = (i as f32 / PARTICLE_COUNT as f32) * stagger_window;
            let age = self.elapsed - spawn_t;
            if age < 0.0 || age > PARTICLE_LIFE {
                continue;
            }
            let progress = age / PARTICLE_LIFE;
            let angle = (i as f32 * 137.5).to_radians();
            let radius_h = 5.0 + progress * 6.0;
            let radius_v = 1.0 + progress * 2.0;
            let drift_up = progress * 2.5;
            let x = center_x + radius_h * angle.cos();
            let y = center_y + radius_v * angle.sin() - drift_up;
            out.push(Particle { x, y, kind });
        }
        out
    }

    fn impact_particles(&self, base: (f32, f32)) -> Vec<Particle> {
        let kind = match self.impact {
            Some(k) => k,
            None => return Vec::new(),
        };
        let start = match impact_start_t(self.kind, self.move_duration) {
            Some(t) => t,
            None => return Vec::new(),
        };
        let age = self.elapsed - start;
        if age < 0.0 || age > IMPACT_DURATION {
            return Vec::new();
        }
        let center_x = self.target_x + 6.0;
        let center_y = base.1 + 2.0;
        let stagger = (IMPACT_DURATION - IMPACT_PARTICLE_LIFE).max(0.0);
        let mut out = Vec::with_capacity(IMPACT_PARTICLE_COUNT);
        for i in 0..IMPACT_PARTICLE_COUNT {
            let spawn_t = (i as f32 / IMPACT_PARTICLE_COUNT as f32) * stagger;
            let p_age = age - spawn_t;
            if p_age < 0.0 || p_age > IMPACT_PARTICLE_LIFE {
                continue;
            }
            let p_progress = p_age / IMPACT_PARTICLE_LIFE;
            let angle = (i as f32 * 137.5 + 30.0).to_radians();
            let radius = 1.0 + p_progress * 2.5;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin() * 0.6 - p_progress * 1.5;
            out.push(Particle { x, y, kind });
        }
        out
    }

    fn trail_particles(&self, base: (f32, f32)) -> Vec<Particle> {
        let kind = match self.trail {
            Some(k) => k,
            None => return Vec::new(),
        };
        let now = self.progress();
        if now > 0.5 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(TRAIL_SAMPLES);
        for i in 1..=TRAIL_SAMPLES {
            let past = now - (i as f32) * TRAIL_STEP;
            if past < 0.0 {
                break;
            }
            let (px, py) = self.crab_position_at(past, base);
            let cx = px + CRAB_HALF_WIDTH;
            let cy = py + CRAB_HALF_HEIGHT;
            let jitter_x = ((i * 17) % 5) as f32 - 2.0;
            let jitter_y = ((i * 31) % 3) as f32 - 1.0;
            out.push(Particle {
                x: cx + jitter_x * 0.6,
                y: cy + jitter_y * 0.5,
                kind,
            });
        }
        out
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub kind: ParticleKind,
}

fn trail_for(kind: AnimationKind, element: Element) -> Option<ParticleKind> {
    if !matches!(kind, AnimationKind::Jump | AnimationKind::Dash) {
        return None;
    }
    match element {
        Element::Fire => Some(ParticleKind::FireSpark),
        Element::Water => Some(ParticleKind::WaterDroplet),
        Element::Ice => Some(ParticleKind::IceShard),
        Element::Electric => Some(ParticleKind::ElectricSpark),
        Element::Ground => Some(ParticleKind::GroundDust),
        Element::Flying => Some(ParticleKind::FlyingWisp),
        Element::Psychic => Some(ParticleKind::PsychicSpark),
        Element::Normal => None,
    }
}

fn impact_for(kind: AnimationKind, element: Element, effect: &Effect) -> Option<ParticleKind> {
    if !matches!(effect, Effect::Damage(_)) {
        return None;
    }
    if !matches!(
        kind,
        AnimationKind::Jump | AnimationKind::Dash | AnimationKind::Throw(_)
    ) {
        return None;
    }
    Some(match element {
        Element::Fire => ParticleKind::FireSpark,
        Element::Water => ParticleKind::WaterDroplet,
        Element::Ice => ParticleKind::IceShard,
        Element::Electric => ParticleKind::ElectricSpark,
        Element::Ground => ParticleKind::GroundDust,
        Element::Flying => ParticleKind::FlyingWisp,
        Element::Psychic => ParticleKind::PsychicSpark,
        Element::Normal => ParticleKind::NormalHit,
    })
}

fn impact_start_t(kind: AnimationKind, move_duration: f32) -> Option<f32> {
    match kind {
        AnimationKind::Jump | AnimationKind::Dash => Some(move_duration * 0.5),
        AnimationKind::Throw(_) => Some(move_duration),
        AnimationKind::SelfCast(_) => None,
    }
}
