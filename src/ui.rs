use crate::app::App;
use ratatui::{
    layout::Offset,
    prelude::{Frame, Rect, Stylize},
    widgets::{Block, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame) {
    let h = frame.size().height as i32;
    let w = frame.size().width as i32;

    let bottom_area = frame
        .size()
        .offset(Offset { x: 0, y: h - 4 })
        .intersection(frame.size());

    let top_area = Rect::new(0, 0, frame.size().width, (h - 4) as u16).intersection(frame.size());
    let top_help_area = Rect::new((w - (w / 3)) as u16, 0, (w / 3) as u16, top_area.height)
        .intersection(top_area.clone());
    let top_chat_area =
        Rect::new(0, 0, (w - (w / 3)) as u16, top_area.height).intersection(top_area.clone());
    frame.render_widget(
        Paragraph::new(app.show_keys("\n"))
            .block(Block::bordered().title(format!("INFO: {}", app.show_current_mode())))
            .green()
            .on_black(),
        top_help_area,
    );
    frame.render_widget(
        Paragraph::new(format!("{}", app.show_logs()))
            .block(Block::bordered().title("LOGS"))
            .green()
            .on_black(),
        top_chat_area,
    );
    frame.render_widget(
        Paragraph::new(format!("{}", app.buffer))
            .block(Block::bordered().title("MSG"))
            .green()
            .on_black(),
        bottom_area,
    );
}
