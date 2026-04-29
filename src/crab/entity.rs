use super::mood::Mood;
use rand::Rng;

const GRAVITY: f32 = 0.1;
const GROUND_FRICTION: f32 = 0.92;
const AIR_FRICTION: f32 = 0.98;

const JUMP_STRENGTH_CELEBRATION: f32 = 2.2;
const JUMP_STRENGTH_ECSTATIC: f32 = 1.8;
const JUMP_STRENGTH_HAPPY: f32 = 1.4;
const JUMP_STRENGTH_NEUTRAL: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

pub struct Eyes;

impl Eyes {
    pub const NEUTRAL: &'static str = "o o";
    pub const HAPPY: &'static str = "^ ^";
    pub const SAD: &'static str = "- -";
    pub const HUNGRY: &'static str = "T T";
    pub const ECSTATIC: &'static str = "* *";
}

pub struct Mouths;

impl Mouths {
    pub const NEUTRAL: &'static str = "-";
    pub const HAPPY: &'static str = "u";
    pub const SAD: &'static str = "n";
    pub const HUNGRY: &'static str = "~";
    pub const ECSTATIC: &'static str = "w";
}

pub struct BodyTemplates;

impl BodyTemplates {
    pub const STANDING_RIGHT: &'static str = r#"    _~^~^~_
\) /  {eyes}  \ (/
  '_   {mouth}   _'
  \ '-----' /"#;

    pub const STANDING_LEFT: &'static str = r#"    _~^~^~_
(\ /  {eyes}  \ ()
  '_   {mouth}   _'
  / '-----' \"#;

    pub const WALKING_RIGHT: &'static str = r#"    _~^~^~_
\) /  {eyes}  \ (/
  '_   {mouth}   _'
  / '-----' \"#;

    pub const WALKING_LEFT: &'static str = r#"    _~^~^~_
(\ /  {eyes}  \ ()
  '_   {mouth}   _'
  \ '-----' /"#;

    pub const CLAPPING_RIGHT: &'static str = r#"    _~^~^~_
\/ /  {eyes}  \ \/
  '_   {mouth}   _'
  \ '-----' /"#;

    pub const CLAPPING_LEFT: &'static str = r#"    _~^~^~_
|| /  {eyes}  \ ||
  '_   {mouth}   _'
  / '-----' \"#;

    pub const BEGGING_RIGHT: &'static str = r#"    _~^~^~_
\\ /  {eyes}  \ //
  '_   {mouth}   _'
  \ '-----' /"#;

    pub const BEGGING_LEFT: &'static str = r#"    _~^~^~_
// /  {eyes}  \ \\
  '_   {mouth}   _'
  / '-----' \"#;

    pub const ECSTATIC_1: &'static str = r#"   ()_~^~^~_()
    /  {eyes}  \
   '_   {mouth}   _'
  \\ '-----' //"#;

    pub const ECSTATIC_2: &'static str = r#"   \/_~^~^~_\/
    /  {eyes}  \
   '_   {mouth}   _'
  // '-----' \\"#;
}

pub fn build_frame(body: &str, eyes: &str, mouth: &str) -> String {
    body.replace("{eyes}", eyes).replace("{mouth}", mouth)
}

pub struct Crab {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub direction: Direction,
    pub mood: Mood,
    pub happiness: u8,
    frame_index: usize,
    animation_timer: f32,
    pub celebrating: bool,
    celebration_timer: f32,
    rng: rand::rngs::ThreadRng,
    pub movement_frozen: bool,
    pub is_grounded: bool,
    ground_y: f32,
    jump_cooldown: f32,
    celebration_jump_done: bool,
    pub anchor_x: Option<f32>,
}

impl Crab {
    pub fn new(position: (f32, f32), happiness: u8) -> Self {
        let mut rng = rand::thread_rng();
        let direction = if rng.gen_bool(0.5) {
            Direction::Right
        } else {
            Direction::Left
        };

        Self {
            position,
            velocity: (0.0, 0.0),
            direction,
            mood: Mood::from_happiness(happiness),
            happiness,
            frame_index: 0,
            animation_timer: 0.0,
            celebrating: false,
            celebration_timer: 0.0,
            rng,
            movement_frozen: false,
            is_grounded: true,
            ground_y: position.1,
            jump_cooldown: 0.0,
            celebration_jump_done: false,
            anchor_x: None,
        }
    }

    pub fn anchor_at(&mut self, x: f32) {
        self.anchor_x = Some(x);
        self.position.0 = x;
        self.velocity.0 = 0.0;
        self.direction = Direction::Right;
    }

    pub fn update(&mut self, dt: f32, bounds: (f32, f32)) {
        self.mood = Mood::from_happiness(self.happiness);

        let frame_height = 4.0;
        let new_ground_y = bounds.1 - frame_height - 1.0;

        if self.is_grounded && (new_ground_y - self.ground_y).abs() > 0.5 {
            self.position.1 = new_ground_y;
        }

        self.ground_y = new_ground_y;

        if self.celebrating {
            self.celebration_timer -= dt;
            if self.celebration_timer <= 0.0 {
                self.celebrating = false;
                self.celebration_jump_done = false;
            }
        }

        let speed_mult = if self.celebrating || !self.is_grounded {
            2.5
        } else {
            self.mood.animation_speed()
        };

        self.animation_timer += dt * speed_mult;
        if self.animation_timer >= 0.3 {
            self.animation_timer = 0.0;
            self.frame_index = (self.frame_index + 1) % 4;
        }

        if self.jump_cooldown > 0.0 {
            self.jump_cooldown -= dt;
        }

        if self.movement_frozen {
            return;
        }

        if self.celebrating && !self.celebration_jump_done && self.is_grounded {
            let strength = self.randomize_jump_strength(JUMP_STRENGTH_CELEBRATION);
            self.jump(strength);
            self.celebration_jump_done = true;
        }

        if self.is_grounded && !self.celebrating {
            let jump_chance = match self.mood {
                Mood::Ecstatic => 0.015,
                Mood::Happy => 0.004,
                Mood::Neutral => 0.001,
                Mood::Sad | Mood::Hungry => 0.0,
            };

            if self.rng.gen::<f32>() < jump_chance {
                let strength = match self.mood {
                    Mood::Ecstatic => JUMP_STRENGTH_ECSTATIC,
                    Mood::Happy => JUMP_STRENGTH_HAPPY,
                    Mood::Neutral => JUMP_STRENGTH_NEUTRAL,
                    _ => 0.0,
                };
                if strength > 0.0 {
                    let strength = self.randomize_jump_strength(strength);
                    self.jump(strength);
                }
            }
        }

        let move_chance = match self.mood {
            Mood::Ecstatic => 0.05,
            Mood::Happy => 0.03,
            Mood::Neutral => 0.02,
            Mood::Sad => 0.01,
            Mood::Hungry => 0.005,
        };

        if self.is_grounded && self.anchor_x.is_none() && self.rng.gen::<f32>() < move_chance {
            let base_speed = match self.mood {
                Mood::Ecstatic => 1.5,
                Mood::Happy => 1.0,
                Mood::Neutral => 0.5,
                Mood::Sad => 0.3,
                Mood::Hungry => 0.1,
            };

            self.velocity.0 = self.rng.gen_range(-base_speed..base_speed);

            if self.velocity.0 > 0.1 {
                self.direction = Direction::Right;
            } else if self.velocity.0 < -0.1 {
                self.direction = Direction::Left;
            }
        }

        if !self.is_grounded {
            self.velocity.1 += GRAVITY * dt * 60.0;
        }

        let friction = if self.is_grounded {
            GROUND_FRICTION
        } else {
            AIR_FRICTION
        };
        self.velocity.0 *= friction;

        self.position.0 += self.velocity.0;
        self.position.1 += self.velocity.1;

        if self.position.1 >= self.ground_y {
            self.position.1 = self.ground_y;
            self.velocity.1 = 0.0;
            self.is_grounded = true;
        } else {
            self.is_grounded = false;
        }

        if self.position.1 < 0.0 {
            self.position.1 = 0.0;
            self.velocity.1 = 0.0;
        }

        let frame_width = 20.0;

        if self.position.0 < 0.0 {
            self.position.0 = 0.0;
            self.velocity.0 = self.velocity.0.abs();
            self.direction = Direction::Right;
        } else if self.position.0 + frame_width > bounds.0 {
            self.position.0 = bounds.0 - frame_width;
            self.velocity.0 = -self.velocity.0.abs();
            self.direction = Direction::Left;
        }

        if let Some(anchor) = self.anchor_x {
            self.position.0 = anchor;
            self.velocity.0 = 0.0;
            self.direction = Direction::Right;
        }
    }

    pub fn jump(&mut self, strength: f32) {
        if self.is_grounded && self.jump_cooldown <= 0.0 {
            self.velocity.1 = -strength;
            self.is_grounded = false;
            self.jump_cooldown = 0.3;
        }
    }

    fn randomize_jump_strength(&mut self, base: f32) -> f32 {
        let variance = self.rng.gen_range(0.6..0.95);
        (base * variance).max(0.7)
    }

    pub fn get_frame(&self) -> String {
        let is_moving = self.velocity.0.abs() > 0.05;
        let is_jumping = !self.is_grounded;

        if is_jumping || self.celebrating || self.mood == Mood::Ecstatic {
            let body = if self.frame_index % 2 == 0 {
                BodyTemplates::ECSTATIC_1
            } else {
                BodyTemplates::ECSTATIC_2
            };
            return build_frame(body, Eyes::ECSTATIC, Mouths::ECSTATIC);
        }

        let (eyes, mouth) = match self.mood {
            Mood::Ecstatic => (Eyes::ECSTATIC, Mouths::ECSTATIC),
            Mood::Happy => (Eyes::HAPPY, Mouths::HAPPY),
            Mood::Neutral => (Eyes::NEUTRAL, Mouths::NEUTRAL),
            Mood::Sad => (Eyes::SAD, Mouths::SAD),
            Mood::Hungry => (Eyes::HUNGRY, Mouths::HUNGRY),
        };

        let body = match self.mood {
            Mood::Ecstatic => {
                if self.frame_index % 2 == 0 {
                    BodyTemplates::ECSTATIC_1
                } else {
                    BodyTemplates::ECSTATIC_2
                }
            }
            Mood::Happy => {
                if is_moving {
                    if self.direction == Direction::Right {
                        if self.frame_index % 2 == 0 {
                            BodyTemplates::STANDING_RIGHT
                        } else {
                            BodyTemplates::WALKING_RIGHT
                        }
                    } else if self.frame_index % 2 == 0 {
                        BodyTemplates::STANDING_LEFT
                    } else {
                        BodyTemplates::WALKING_LEFT
                    }
                } else if self.frame_index % 4 == 0 {
                    if self.direction == Direction::Right {
                        BodyTemplates::CLAPPING_RIGHT
                    } else {
                        BodyTemplates::CLAPPING_LEFT
                    }
                } else if self.direction == Direction::Right {
                    BodyTemplates::STANDING_RIGHT
                } else {
                    BodyTemplates::STANDING_LEFT
                }
            }
            Mood::Neutral => {
                if is_moving {
                    if self.direction == Direction::Right {
                        if self.frame_index % 2 == 0 {
                            BodyTemplates::STANDING_RIGHT
                        } else {
                            BodyTemplates::WALKING_RIGHT
                        }
                    } else if self.frame_index % 2 == 0 {
                        BodyTemplates::STANDING_LEFT
                    } else {
                        BodyTemplates::WALKING_LEFT
                    }
                } else if self.direction == Direction::Right {
                    BodyTemplates::STANDING_RIGHT
                } else {
                    BodyTemplates::STANDING_LEFT
                }
            }
            Mood::Sad => {
                if self.direction == Direction::Right {
                    BodyTemplates::STANDING_RIGHT
                } else {
                    BodyTemplates::STANDING_LEFT
                }
            }
            Mood::Hungry => {
                if self.frame_index % 2 == 0 {
                    if self.direction == Direction::Right {
                        BodyTemplates::BEGGING_RIGHT
                    } else {
                        BodyTemplates::BEGGING_LEFT
                    }
                } else if self.direction == Direction::Right {
                    BodyTemplates::STANDING_RIGHT
                } else {
                    BodyTemplates::STANDING_LEFT
                }
            }
        };

        build_frame(body, eyes, mouth)
    }

    pub fn celebrate(&mut self) {
        self.celebrating = true;
        self.celebration_timer = 3.0;
    }

    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        if self.celebrating {
            Color::LightMagenta
        } else {
            match self.mood {
                Mood::Ecstatic => Color::Rgb(255, 100, 100),
                Mood::Happy => Color::Rgb(255, 120, 80),
                Mood::Neutral => Color::Rgb(220, 100, 80),
                Mood::Sad => Color::Rgb(180, 80, 80),
                Mood::Hungry => Color::Rgb(150, 60, 60),
            }
        }
    }
}
