use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

pub fn render_element(
    frame: &mut Frame,
    content: &[String],
    x: i32,
    y: i32,
    color: Color,
    area: Rect,
) {
    for (i, line) in content.iter().enumerate() {
        let y_pos = y + i as i32;
        if y_pos < 0 || y_pos >= area.height as i32 {
            continue;
        }

        if x >= area.width as i32 {
            continue;
        }

        let x_start = x.max(0) as u16;
        let max_width = area.width.saturating_sub(x_start) as usize;
        if max_width == 0 {
            continue;
        }

        let line_start = if x < 0 { (-x) as usize } else { 0 };
        if line_start >= line.len() {
            continue;
        }

        let visible = &line[line_start..];
        let width = visible.len().min(max_width) as u16;
        let visible = &visible[..width as usize];

        let line_area = Rect {
            x: area.x + x_start,
            y: area.y + y_pos as u16,
            width,
            height: 1,
        };

        let line_widget = Paragraph::new(visible.to_string()).style(Style::default().fg(color));
        frame.render_widget(line_widget, line_area);
    }
}
