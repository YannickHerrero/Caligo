use crate::crab::Crab;
use crate::environment::{Environment, GroundStyle, TimeOfDay};
use crate::fight::{Animation, Enemy};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::Paragraph,
    Frame,
};

use super::helpers::render_element;

pub fn render_crab(
    frame: &mut Frame,
    crab: &Crab,
    area: Rect,
    position_override: Option<(f32, f32)>,
) {
    let crab_frame = crab.get_frame();
    let color = crab.color();

    let (px, py) = position_override.unwrap_or(crab.position);
    let x_offset = px.max(0.0) as u16;
    let y_offset = py.max(0.0) as u16;

    let lines: Vec<Line> = crab_frame
        .lines()
        .map(|line| {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let crab_text = Text::from(lines);

    let crab_area = Rect {
        x: area.x + x_offset.min(area.width.saturating_sub(22)),
        y: area.y + y_offset.min(area.height.saturating_sub(5)),
        width: 22.min(area.width),
        height: 4.min(area.height),
    };

    let paragraph = Paragraph::new(crab_text);
    frame.render_widget(paragraph, crab_area);
}

pub fn render_environment_background(frame: &mut Frame, env: &Environment, area: Rect) {
    let celestial_color = match env.time_of_day {
        TimeOfDay::Morning => Color::Yellow,
        TimeOfDay::Day => Color::Yellow,
        TimeOfDay::Evening => Color::Rgb(255, 200, 100),
        TimeOfDay::Night => Color::White,
    };

    if env.time_of_day == TimeOfDay::Night {
        for star in &env.stars {
            if star.x < area.width && star.y < area.height {
                let star_area = Rect {
                    x: area.x + star.x,
                    y: area.y + star.y,
                    width: 1,
                    height: 1,
                };
                let star_char =
                    Paragraph::new(star.char.to_string()).style(Style::default().fg(Color::White));
                frame.render_widget(star_char, star_area);
            }
        }
    }

    if let Some((x, y)) = env.sun_position() {
        render_element(
            frame,
            &crate::environment::elements::SUN
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            x,
            y,
            celestial_color,
            area,
        );
    }

    if let Some((x, y)) = env.moon_position() {
        render_element(
            frame,
            &crate::environment::elements::MOON_SMALL
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            x,
            y,
            Color::Rgb(200, 200, 220),
            area,
        );
    }

    let cloud_color = if env.time_of_day == TimeOfDay::Night {
        Color::DarkGray
    } else {
        Color::Gray
    };
    for cloud in &env.clouds {
        if env.time_of_day == TimeOfDay::Night && !cloud.night_visible {
            continue;
        }
        let cloud_x = cloud.x.round() as i32;
        if cloud_x >= area.width as i32 || cloud_x + cloud.width as i32 <= 0 {
            continue;
        }

        render_element(
            frame,
            &cloud.content,
            cloud_x,
            cloud.y as i32,
            cloud_color,
            area,
        );
    }
}

pub fn render_ground(frame: &mut Frame, env: &Environment, area: Rect) {
    if area.height == 0 {
        return;
    }

    let ground_y = area.y + area.height.saturating_sub(1);

    let ground_color = match env.time_of_day {
        TimeOfDay::Night => match env.ground_style {
            GroundStyle::Beach => Color::Rgb(82, 72, 52),
            GroundStyle::Garden => Color::Rgb(28, 64, 40),
            GroundStyle::Rocky => Color::Rgb(70, 74, 78),
            GroundStyle::Minimal => Color::Rgb(48, 72, 44),
        },
        _ => match env.ground_style {
            GroundStyle::Beach => Color::Rgb(200, 183, 132),
            GroundStyle::Garden => Color::Rgb(46, 128, 72),
            GroundStyle::Rocky => Color::Rgb(126, 132, 138),
            GroundStyle::Minimal => Color::Rgb(96, 146, 78),
        },
    };

    let ground_display: String = env.ground_line.chars().take(area.width as usize).collect();

    let ground_area = Rect {
        x: area.x,
        y: ground_y,
        width: area.width,
        height: 1,
    };

    let ground_widget = Paragraph::new(ground_display).style(Style::default().fg(ground_color));
    frame.render_widget(ground_widget, ground_area);
}

pub fn render_enemy(frame: &mut Frame, enemy: &Enemy, area: Rect) {
    let sprite_width = enemy
        .sprite
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0) as i32;
    let sprite_height = enemy.sprite.len() as i32;

    if sprite_width == 0 || sprite_height == 0 {
        return;
    }

    let x = area.width as i32 - sprite_width - 4;
    let y = area.height as i32 - sprite_height - 1;

    render_element(frame, &enemy.sprite, x, y, enemy.color, area);
}

pub fn render_projectile(frame: &mut Frame, anim: &Animation, ground_y: f32, area: Rect) {
    let Some((x, y)) = anim.projectile_position(ground_y) else {
        return;
    };
    if x < 0.0 || y < 0.0 {
        return;
    }
    let xi = x.round() as u16;
    let yi = y.round() as u16;
    if xi >= area.width || yi >= area.height {
        return;
    }
    let cell = Rect {
        x: area.x + xi,
        y: area.y + yi,
        width: 1,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new("●").style(
            Style::default()
                .fg(Color::Rgb(140, 220, 255))
                .add_modifier(Modifier::BOLD),
        ),
        cell,
    );
}
