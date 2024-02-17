use crate::app::App;
use ratatui::{
    layout::Offset,
    prelude::{Frame, Rect, Stylize},
    widgets::{Block, Paragraph},
};

fn h_split(frame: &mut Frame, rows: usize) -> [Rect; 2] {
    let h = frame.size().height as i32;

    let top_area = frame
        .size()
        .offset(Offset {
            x: 0,
            y: rows as i32,
        })
        .intersection(frame.size())
        .offset(Offset {
            x: 0,
            y: -(rows as i32),
        });

    let bottom_area = frame
        .size()
        .offset(Offset {
            x: 0,
            y: h - (rows as i32),
        })
        .intersection(frame.size());

    [top_area, bottom_area]
}

fn v_split(rect: Rect) -> [Rect; 2] {
    let w = rect.width as i32;
    let left_area = rect
        .offset(Offset { x: w / 3, y: 0 })
        .intersection(rect.clone())
        .offset(Offset { x: -w / 3, y: 0 });
    let right_area = rect
        .offset(Offset {
            x: 1 + w - (w / 3),
            y: 0,
        })
        .intersection(rect.clone());

    [left_area, right_area]
}

fn top_help_widget(app: &App) -> Paragraph {
    Paragraph::new(app.show_keys("\n"))
        .block(Block::bordered().title(format!("INFO: {}", app.show_current_mode())))
        .green()
        .on_black()
}

fn chat_log_widget(app: &App) -> Paragraph {
    Paragraph::new(format!("{}", app.show_logs()))
        .block(Block::bordered().title("LOGS"))
        .green()
        .on_black()
}

fn textarea_widget(app: &App) -> Paragraph {
    Paragraph::new(app.render_buf_styled()).block(Block::bordered().title("MSG"))
}

pub fn render(app: &App, frame: &mut Frame) {
    let [top_area, bottom_area] = h_split(frame, 6);
    let [top_left, top_right] = v_split(top_area);

    frame.render_widget(top_help_widget(app), top_right);
    frame.render_widget(chat_log_widget(app), top_left);
    frame.render_widget(textarea_widget(app), bottom_area);
}
