use crate::{app::App, chat_log::LogStyle};
use ratatui::{
    layout::Offset,
    prelude::{Frame, Rect, Stylize},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Padding, Paragraph, Wrap},
};

fn h_split(frame: &Rect, rows: usize) -> [Rect; 2] {
    let h = frame.height as i32;

    let top_area = frame
        .offset(Offset {
            x: 0,
            y: rows as i32,
        })
        .intersection(frame.clone())
        .offset(Offset {
            x: 0,
            y: -(rows as i32),
        });

    let bottom_area = frame
        .offset(Offset {
            x: 0,
            y: h - (rows as i32),
        })
        .intersection(frame.clone());

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
            x: w - (w / 3),
            y: 0,
        })
        .intersection(rect.clone());

    [left_area, right_area]
}

fn top_help_widget(app: &App) -> Paragraph {
    Paragraph::new(app.render_keymap())
        .block(
            Block::bordered()
                .title(Span::styled(
                    format!("INFO: {}", app.show_current_mode()),
                    Style::new().white().on_black(),
                ))
                .padding(Padding::left(1)),
        )
        .green()
        .on_black()
}

fn room_info_widget(app: &App) -> Paragraph {
    let block = Block::bordered().title(Span::styled(
        format!("ROOM: {}", app.room_state.room_name),
        Style::new().fg(Color::White),
    ));

    let mut text = "".to_string();
    let mut prefix: String = "".into();
    for username in &app.room_state.occupants {
        text = text + &(prefix + username);
        prefix = "\n".into();
    }

    Paragraph::new(text)
        .block(block)
        .green()
        .on_black()
        .wrap(Wrap { trim: false })
}

fn chat_log_widget(app: &App, area: Rect) -> Paragraph {
    let block = Block::bordered().title(Span::styled("LOGS", Style::new().fg(Color::White)));
    let text = app.render_logs(
        (area.height as usize).checked_sub(2).unwrap_or(0),
        &LogStyle::default(),
    );
    Paragraph::new(text)
        .block(block)
        .green()
        .on_black()
        .wrap(Wrap { trim: false })
}

fn textarea_widget(app: &App) -> Paragraph {
    Paragraph::new(app.render_buf_styled())
        .block(Block::bordered().green().on_black().title(Span::styled(
            app.input_area_name(),
            Style::new().fg(Color::White),
        )))
        .white()
        .on_black()
}

pub fn render(app: &App, frame: &mut Frame) {
    let [top_area, bottom_area] = h_split(&frame.size(), 6);
    let [top_left, top_right] = v_split(top_area);
    let [top_top_right, btm_top_right] = h_split(&top_right, (top_right.height / 2) as usize);

    frame.render_widget(top_help_widget(app), top_top_right);
    frame.render_widget(room_info_widget(app), btm_top_right);
    frame.render_widget(chat_log_widget(app, top_left.clone()), top_left);
    frame.render_widget(textarea_widget(app), bottom_area);
}
