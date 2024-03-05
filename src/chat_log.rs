use std::fmt::{Debug, Display};

use chrono::{DateTime, Utc};
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
};

pub struct LogStyle {
    time_style: Style,
    uname_style: Style,
    msg_style: Style,
    delim_style: Style,
    time_fmt: String,
}

impl Default for LogStyle {
    fn default() -> Self {
        Self {
            time_style: Style::new().fg(Color::Gray).bg(Color::Black).italic(),
            uname_style: Style::new().fg(Color::Yellow).bg(Color::Black).bold(),
            msg_style: Style::new().fg(Color::White).bg(Color::Black),
            delim_style: Style::new().fg(Color::Blue).bg(Color::Black),
            time_fmt: "%H:%M:%S".to_string(),
        }
    }
}

impl LogStyle {
    pub fn time(&self) -> Style {
        self.time_style.clone()
    }

    pub fn time_fmt_str(&self) -> &str {
        &self.time_fmt
    }

    pub fn uname(&self) -> Style {
        self.uname_style.clone()
    }

    pub fn delims(&self) -> Style {
        self.delim_style.clone()
    }

    pub fn msg(&self) -> Style {
        self.msg_style.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Log {
    pub ts: DateTime<Utc>,
    pub from: String,
    pub msg: String,
    pub debug: bool,
}

impl Log {
    pub fn new(uname: String, message: String) -> Self {
        Self {
            ts: Utc::now(),
            from: uname,
            msg: message,
            debug: false,
        }
    }

    pub fn new_debug(data: impl Debug) -> Self {
        Self::new("DEBUG".into(), format!("{data:?}")).as_debug()
    }

    pub fn as_debug(mut self) -> Self {
        self.debug = true;

        self
    }

    pub fn at(mut self, dt: DateTime<Utc>) -> Self {
        self.ts = dt;

        self
    }

    pub fn get_ts(&self) -> DateTime<Utc> {
        self.ts.clone()
    }
    pub fn get_msg_body(&self) -> String {
        self.msg.clone()
    }

    #[allow(dead_code)]
    pub fn get_username(&self) -> String {
        self.from.clone()
    }

    pub fn should_render(&self, show_debug: bool) -> bool {
        (!self.debug) || show_debug
    }

    pub fn render(&self, styles: &LogStyle) -> Line {
        Line::default().spans([
            Span::styled("[ ", styles.delims()),
            Span::styled(
                self.ts.format(styles.time_fmt_str()).to_string(),
                styles.time(),
            ),
            Span::styled(" : ", styles.delims()),
            Span::styled(self.get_username(), styles.uname()),
            Span::styled(" ]: ", styles.delims()),
            Span::styled(self.msg.clone(), styles.msg()),
        ])
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} | {} ]: {}",
            self.ts.format("%H:%M:%S").to_string(),
            self.from,
            self.msg
        )
    }
}
