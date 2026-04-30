pub mod elements;

use rand::seq::SliceRandom;
use rand::Rng;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Morning,
    Day,
    Evening,
    Night,
}

impl TimeOfDay {
    pub const ALL: &'static [TimeOfDay] = &[
        TimeOfDay::Morning,
        TimeOfDay::Day,
        TimeOfDay::Evening,
        TimeOfDay::Night,
    ];

    pub fn from_phase(phase: f32) -> Self {
        if phase < 0.2 {
            TimeOfDay::Morning
        } else if phase < 0.45 {
            TimeOfDay::Day
        } else if phase < 0.5 {
            TimeOfDay::Evening
        } else {
            TimeOfDay::Night
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TimeOfDay::Morning => "Morning",
            TimeOfDay::Day => "Day",
            TimeOfDay::Evening => "Evening",
            TimeOfDay::Night => "Night",
        }
    }

    pub fn canonical_phase(&self) -> f32 {
        match self {
            TimeOfDay::Morning => 0.1,
            TimeOfDay::Day => 0.3,
            TimeOfDay::Evening => 0.475,
            TimeOfDay::Night => 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GroundStyle {
    #[default]
    Beach,
    Garden,
    Rocky,
    Minimal,
}

impl GroundStyle {
    pub const ALL: &'static [GroundStyle] = &[
        GroundStyle::Beach,
        GroundStyle::Garden,
        GroundStyle::Rocky,
        GroundStyle::Minimal,
    ];

    pub fn ground_chunks(&self) -> &'static [&'static str] {
        match self {
            GroundStyle::Beach => elements::BEACH_CHUNKS,
            GroundStyle::Garden => elements::GARDEN_CHUNKS,
            GroundStyle::Rocky => elements::ROCKY_CHUNKS,
            GroundStyle::Minimal => elements::MINIMAL_CHUNKS,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            GroundStyle::Beach => "Beach",
            GroundStyle::Garden => "Garden",
            GroundStyle::Rocky => "Rocky",
            GroundStyle::Minimal => "Minimal",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Star {
    pub x: u16,
    pub y: u16,
    pub char: char,
}

#[derive(Debug, Clone)]
pub struct Cloud {
    pub x: f32,
    pub y: u16,
    pub speed: f32,
    pub content: Vec<String>,
    pub width: u16,
    pub night_visible: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub ground_style: GroundStyle,
    pub ground_line: String,
    pub clouds: Vec<Cloud>,
    pub stars: Vec<Star>,
    pub width: u16,
    pub height: u16,
    pub time_of_day: TimeOfDay,
    pub cycle_phase: f32,
    pub cycle_duration: Duration,
}

impl Environment {
    pub fn generate_at(width: u16, height: u16, style: GroundStyle, time: TimeOfDay) -> Self {
        let mut env = Self::generate(width, height, style);
        env.cycle_phase = time.canonical_phase();
        env.time_of_day = time;
        env.stars = if time == TimeOfDay::Night {
            let mut rng = rand::thread_rng();
            Self::generate_stars(width, height, &mut rng)
        } else {
            Vec::new()
        };
        env
    }

    pub fn generate(width: u16, height: u16, style: GroundStyle) -> Self {
        let mut rng = rand::thread_rng();
        let cycle_duration = Duration::from_secs(18 * 60);
        let cycle_phase = 0.0;
        let time_of_day = TimeOfDay::from_phase(cycle_phase);

        let ground_line = Self::generate_ground_line(width, style, &mut rng);
        let clouds = Self::generate_clouds(width, height, &mut rng);
        let stars = if time_of_day == TimeOfDay::Night {
            Self::generate_stars(width, height, &mut rng)
        } else {
            Vec::new()
        };

        Self {
            ground_style: style,
            ground_line,
            clouds,
            stars,
            width,
            height,
            time_of_day,
            cycle_phase,
            cycle_duration,
        }
    }

    fn generate_ground_line(width: u16, style: GroundStyle, rng: &mut impl Rng) -> String {
        if width == 0 {
            return String::new();
        }

        let chunks = style.ground_chunks();
        let mut line = String::new();
        let mut length = 0usize;
        let target = width as usize;

        while length < target {
            let chunk = chunks.choose(rng).unwrap_or(&"..");
            let chunk_len = chunk.chars().count();
            let remaining = target - length;

            if chunk_len <= remaining {
                line.push_str(chunk);
                length += chunk_len;
            } else {
                line.extend(chunk.chars().take(remaining));
                length = target;
            }
        }

        line
    }

    fn generate_clouds(width: u16, height: u16, rng: &mut impl Rng) -> Vec<Cloud> {
        let mut clouds = Vec::new();

        if height < 6 || width < 20 {
            return clouds;
        }

        let cloud_count = rng.gen_range(2..=4);

        for _ in 0..cloud_count {
            let cloud = if rng.gen_bool(0.5) {
                elements::CLOUD_SMALL
            } else {
                elements::CLOUD_LARGE
            };

            let cloud_width = cloud[0].len() as u16;
            let spawn_left = -(cloud_width as f32 * rng.gen_range(1.0..2.5));
            let spawn_right = width as f32 + cloud_width as f32;
            let cloud_x = rng.gen_range(spawn_left..spawn_right);
            let cloud_y = rng.gen_range(0..height / 3);
            let speed = rng.gen_range(0.25..0.7);
            let night_visible = rng.gen_bool(0.6);

            clouds.push(Cloud {
                x: cloud_x,
                y: cloud_y,
                speed,
                content: cloud.iter().map(|s| s.to_string()).collect(),
                width: cloud_width,
                night_visible,
            });
        }

        clouds
    }

    fn generate_stars(width: u16, height: u16, rng: &mut impl Rng) -> Vec<Star> {
        let mut stars = Vec::new();

        let star_count = (width as usize * height as usize) / 40;
        let star_count = star_count.min(30);

        for _ in 0..star_count {
            let max_y = (height * 2 / 3).max(1);
            stars.push(Star {
                x: rng.gen_range(0..width),
                y: rng.gen_range(0..max_y),
                char: *elements::STAR_CHARS.choose(rng).unwrap_or(&'*'),
            });
        }

        stars
    }

    pub fn update_cycle(&mut self, dt: f32, cycle_speed: f32, cloud_speed: f32) {
        let cycle_seconds = self.cycle_duration.as_secs_f32().max(1.0);
        let cycle_dt = dt * cycle_speed;
        let cloud_dt = dt * cloud_speed;
        self.cycle_phase = (self.cycle_phase + (cycle_dt / cycle_seconds)) % 1.0;

        let new_time = TimeOfDay::from_phase(self.cycle_phase);
        if new_time != self.time_of_day {
            let mut rng = rand::thread_rng();
            self.time_of_day = new_time;
            self.stars = if new_time == TimeOfDay::Night {
                Self::generate_stars(self.width, self.height, &mut rng)
            } else {
                Vec::new()
            };
        }

        for cloud in &mut self.clouds {
            cloud.x += cloud.speed * cloud_dt;
            if cloud.x > self.width as f32 + cloud.width as f32 {
                cloud.x = -(cloud.width as f32);
            }
        }
    }

    pub fn sun_position(&self) -> Option<(i32, i32)> {
        if self.cycle_phase >= 0.5 {
            return None;
        }
        Some(self.arc_position(self.cycle_phase * 2.0))
    }

    pub fn moon_position(&self) -> Option<(i32, i32)> {
        if self.cycle_phase < 0.5 {
            return None;
        }
        Some(self.arc_position((self.cycle_phase - 0.5) * 2.0))
    }

    fn arc_position(&self, t: f32) -> (i32, i32) {
        let width = self.width.max(1) as f32;
        let height = self.height.max(1) as f32;
        let left_x = -(width * 0.1).max(3.0);
        let right_x = width * 1.1;
        let base_y = height * 0.25;
        let apex_y = (height * 0.05).max(0.0);
        let arc_height = (base_y - apex_y).max(1.0);

        let x = left_x + (right_x - left_x) * t;
        let y = base_y - arc_height * (std::f32::consts::PI * t).sin();

        (x.round() as i32, y.round() as i32)
    }
}
