use chrono::prelude::*;
use crossterm::event::KeyCode;
use log::info;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::tui_framework::Event;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Mode {
    Navigate,
    Insert,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Hash, Debug, Clone)]
pub enum CaretMotion {
    Character,
    Line,
}

#[derive(Debug, Clone, Hash)]
pub enum Command {
    Reset,
    Quit,
    Capture(char),
    Del(isize),
    MoveCaret(CaretMotion, isize),
    Enter(Mode),
    SendBuffer,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;
        let s = match self {
            Reset => "Reset",
            Quit => "Quit",
            Capture(_) => "",
            MoveCaret(_, _) => "Move Cursor",
            Del(_) => "Delete",
            Enter(Mode::Navigate) => "Enter Navigation Mode",
            Enter(Mode::Insert) => "Enter Insert Mode",
            SendBuffer => "Send Message",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug)]
pub struct Log(pub DateTime<Utc>, pub String, pub String);

impl Log {
    pub fn always_from_string(raw: String) -> Self {
        let (mut metadata, msg) = match raw.split_once(": ") {
            Some((l, r)) => (l.to_string(), r.to_string()),
            None => ("UNKNOWN".into(), raw),
        };

        if metadata.starts_with("[") {
            metadata = metadata.chars().skip(1).collect();
        }
        if metadata.ends_with("]") && metadata.len() > 0 {
            metadata = metadata.chars().take(metadata.len() - 1).collect();
        }

        if let Some((_ts, uname)) = metadata.split_once(" | ") {
            Log(Utc::now(), uname.trim().into(), msg)
        } else {
            Log(Utc::now(), "UNKNOWN".into(), msg)
        }
    }

    pub fn get_ts(&self) -> DateTime<Utc> {
        self.0.clone()
    }
    pub fn get_msg_body(&self) -> String {
        self.2.clone()
    }

    #[allow(dead_code)]
    pub fn get_username(&self) -> String {
        self.1.clone()
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} | {} ]: {}",
            self.0.format("%H-%M-%S").to_string(),
            self.1,
            self.2
        )
    }
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub buffer: Vec<String>,
    pub caret_offset: (usize, usize),
    pub logs: VecDeque<Log>,
    pub mode: Mode,
    pub keymaps: ModalKeyMaps,
    pub username: String,
    pub token: Option<String>,
    pub server_command_sink: Option<UnboundedSender<Event>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            mode: Mode::Navigate,
            buffer: vec!["".into()],
            caret_offset: (1, 1),
            logs: VecDeque::new(),
            keymaps: ModalKeyMaps::default(),
            username: format!("User {}", Utc::now().timestamp_micros() % 1024,),
            token: None,
            server_command_sink: None,
        }
    }

    pub fn set_send_chan(&mut self, chan: UnboundedSender<Event>) {
        self.server_command_sink = Some(chan);
    }

    pub fn map_key(&self, code: KeyCode) -> Option<Command> {
        info!("App mapping key {code:?}");
        self.keymaps.get_cmd(&self.mode, code)
    }

    pub fn show_logs(&self) -> String {
        self.logs
            .iter()
            .rev()
            .map(|l| format!("{}", l))
            .fold("".to_string(), |acc, el| acc + "\n" + &el)
    }

    pub fn show_current_mode(&self) -> String {
        return format!("{}", self.mode);
    }

    pub fn show_keys(&self, sep: &str) -> String {
        self.keymaps.show(&self.mode, sep)
    }

    pub fn get_caret_2d(&self) -> (usize, usize) {
        self.caret_offset
    }

    pub fn set_caret_2d(&mut self, row: usize, col: usize) {
        self.caret_offset.0 = row.clamp(1, self.buffer.len());
        self.caret_offset.1 = col.clamp(1, self.buffer[row.checked_sub(1).unwrap_or(0)].len() + 1);
    }

    pub fn render_buf(&self) -> String {
        self.buffer.iter().fold("".to_string(), |acc, el| acc + &el)
    }

    pub fn split_current_at_caret(&self) -> (String, String) {
        let (row, col) = self.get_caret_2d();

        let buf_line = self.buffer[row.checked_sub(1).unwrap_or(0)].clone();
        let (pre, post) = if buf_line.len() > 0 {
            buf_line.split_at(col - 1)
        } else {
            (buf_line.as_str(), "")
        };
        let pre = pre.to_string();

        let (mut up_to, mut caret_and_beyond) = ("".to_string(), "".to_string());
        up_to = up_to + &pre;

        let post = post.to_string();
        caret_and_beyond = caret_and_beyond + &post;

        (up_to, caret_and_beyond)
    }

    pub fn render_buf_styled(&self) -> Line {
        let mut line_vec: Vec<Span> = vec![];
        let (row, col) = self.get_caret_2d();
        let preceding_chunk = self
            .buffer
            .iter()
            .take(row - 1)
            .fold(String::from(""), |acc, el| acc + &el);

        if preceding_chunk.len() > 0 {
            line_vec.push(Span::raw(preceding_chunk));
        }

        let buf_line = self.buffer[row.checked_sub(1).unwrap_or(0)].clone();
        let (pre, post) = if buf_line.len() > 0 {
            buf_line.split_at(col - 1)
        } else {
            (buf_line.as_str(), "")
        };
        let post = post.to_string();
        let pre = pre.to_string();
        line_vec.push(Span::raw(pre));

        let blinkin = Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::SLOW_BLINK);
        let highlighted = match post.len() {
            0 => Span::styled(" ".to_string(), blinkin),
            _ => Span::styled(post.clone().chars().take(1).collect::<String>(), blinkin),
        };

        line_vec.push(highlighted);

        let rest_of_line = Span::raw(post.clone().chars().skip(1).collect::<String>());
        line_vec.push(rest_of_line);

        let subsequent_lines = self
            .buffer
            .iter()
            .skip(row)
            .fold(String::from(""), |acc, el| acc + &el);
        line_vec.push(Span::raw(subsequent_lines));

        Line::from(line_vec)
    }

    pub fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::Quit => {
                self.should_quit = true;
            }
            Command::Reset => {
                self.buffer = vec!["".into()];
                self.caret_offset = (1, 1);
            }
            Command::Capture(c) => {
                self.handle_capture(c);
            }
            Command::Enter(mode) => {
                self.switch_mode(mode);
            }
            Command::SendBuffer => {
                self.handle_send();
            }
            Command::MoveCaret(motion, amount) => {
                self.handle_caret_move(motion, amount);
            }
            Command::Del(offset) => self.handle_deletion(offset),
        };
        info!("Caret: {:?}", self.caret_offset);
    }

    fn handle_deletion(&mut self, offset: isize) {
        let (pre, post) = self.split_current_at_caret();
        let (row, col) = self.get_caret_2d();
        let line_with_removal = match offset.signum() < 0 {
            false => {
                if post.len() >= 1 {
                    pre + &String::from(post.chars().skip(1).collect::<String>())
                } else {
                    pre + &post
                }
            }
            true => {
                if pre.len() >= 1 {
                    self.set_caret_2d(row, col - 1);
                    String::from(pre.chars().take(pre.len() - 1).collect::<String>() + &post)
                } else {
                    pre + &post
                }
            }
        };
        self.buffer[row.checked_sub(1).unwrap_or(0)] = line_with_removal;
    }

    fn handle_caret_move(&mut self, motion: CaretMotion, amount: isize) {
        let (row, col) = self.get_caret_2d();
        let new_caret = match motion {
            CaretMotion::Character => (row, (col as isize + amount).max(0) as usize),
            CaretMotion::Line => ((row as isize + amount).max(0) as usize, col),
        };
        self.set_caret_2d(new_caret.0, new_caret.1);
    }

    fn switch_mode(&mut self, mode: Mode) {
        self.mode = mode;
        match self.mode {
            Mode::Insert => {
                let (r, c) = self.get_caret_2d();
                self.set_caret_2d(r, c);
            }
            Mode::Navigate => {}
        }
    }

    pub fn handle_send(&mut self) {
        let chat_log = Log(Utc::now(), self.username.clone(), self.render_buf());
        if let Some(ref chan) = self.server_command_sink {
            let Ok(_) = chan.send(Event::Send {
                token: self.token.clone(),
                username: self.username.clone(),
                timestamp: chat_log.get_ts(),
                contents: chat_log.get_msg_body(),
            }) else {
                return;
            };
        }
        self.push_log(chat_log);
        self.buffer = vec!["".into()];
        self.caret_offset = (1, 1);
    }

    pub fn push_log(&mut self, log: Log) {
        self.logs.push_front(log);
        if self.logs.len() > 100 {
            self.logs.pop_back();
        }
    }

    pub fn store_token(&mut self, token: String) {
        self.token = Some(token);
    }

    fn handle_capture(&mut self, c: char) {
        let (row, col) = self.get_caret_2d();
        let mut buf_line = self.buffer[row.checked_sub(1).unwrap_or(0)].clone();
        if col < buf_line.len() {
            let post = buf_line.split_off(col);
            buf_line.push(c);
            buf_line = buf_line + &post;
        } else {
            buf_line.push(c);
        }
        self.buffer[row.checked_sub(1).unwrap_or(0)] = buf_line;
        self.caret_offset = (self.caret_offset.0, self.caret_offset.1 + 1);
    }
}

type KeyCheck = dyn Fn(KeyCode) -> Option<Command>;

#[allow(dead_code)]
pub enum KeyBinds {
    Explicit(KeyCode, Command),
    Logical(Box<KeyCheck>),
    NoMap,
}

impl std::fmt::Debug for KeyBinds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explicit(k, c) => write!(f, "KeyBinds::Explicit({k:?}, {c:?})"),
            Self::Logical(_) => write!(f, "KeyBinds::Logical(fn)"),
            Self::NoMap => write!(f, "KeyBinds::NoMap"),
        }
    }
}

impl Display for KeyBinds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explicit(KeyCode::Char(c), cmd) => write!(f, "{c}\t -> {cmd}"),
            Self::Explicit(KeyCode::Enter, cmd) => write!(f, "󰌑\t -> {cmd}"),
            Self::Explicit(KeyCode::Esc, cmd) => write!(f, "Esc\t -> {cmd}"),
            Self::Explicit(KeyCode::Delete, cmd) => write!(f, "Del\t -> {cmd}"),
            Self::Explicit(KeyCode::Left, cmd) => write!(f, "←/→\t -> {cmd}"),
            _ => write!(f, ""),
        }
    }
}

impl KeyBinds {
    pub fn check(&self, c: KeyCode) -> Option<Command> {
        match self {
            Self::Explicit(code, ref command) if code.clone() == c => Some(command.clone()),
            Self::Logical(check_fn) => check_fn(c),
            Self::NoMap => None,
            _ => None,
        }
    }

    pub fn capture() -> Self {
        Self::Logical(Box::new(|keycode: KeyCode| match keycode {
            KeyCode::Char(c) => Some(Command::Capture(c)),
            _ => None,
        }))
    }
}

#[derive(Debug)]
pub struct ModalKeyMaps {
    keymaps: HashMap<Mode, Vec<KeyBinds>>,
}

impl ModalKeyMaps {
    fn get_cmd(&self, mode: &Mode, code: KeyCode) -> Option<Command> {
        if let Some(binds) = self.keymaps.get(&mode) {
            for binding in binds {
                if let Some(cmd) = binding.check(code) {
                    return Some(cmd);
                }
            }
        }

        return None;
    }

    fn show(&self, mode: &Mode, sep: &str) -> String {
        let mut result = "".to_string();
        if let Some(binds) = self.keymaps.get(mode) {
            for item in binds {
                if format!("{}", item).len() > 0 {
                    result = result + sep + &format!("{item}");
                }
            }
        }

        result
    }
}

impl Default for ModalKeyMaps {
    fn default() -> Self {
        Self {
            keymaps: HashMap::from([
                (
                    Mode::Navigate,
                    vec![
                        KeyBinds::Explicit(KeyCode::Char('i'), Command::Enter(Mode::Insert)),
                        KeyBinds::Explicit(KeyCode::Char('q'), Command::Quit),
                        KeyBinds::Explicit(KeyCode::Char('r'), Command::Reset),
                    ],
                ),
                (
                    Mode::Insert,
                    vec![
                        // leave insert mode
                        KeyBinds::Explicit(KeyCode::Esc, Command::Enter(Mode::Navigate)),
                        // send message
                        KeyBinds::Explicit(KeyCode::Enter, Command::SendBuffer),
                        // Caret controls
                        KeyBinds::Explicit(
                            KeyCode::Left,
                            Command::MoveCaret(CaretMotion::Character, -1),
                        ),
                        KeyBinds::Explicit(
                            KeyCode::Right,
                            Command::MoveCaret(CaretMotion::Character, 1),
                        ),
                        KeyBinds::Explicit(KeyCode::Up, Command::MoveCaret(CaretMotion::Line, -1)),
                        KeyBinds::Explicit(KeyCode::Down, Command::MoveCaret(CaretMotion::Line, 1)),
                        // text input
                        KeyBinds::capture(),
                        // deletion
                        KeyBinds::Explicit(KeyCode::Backspace, Command::Del(-1)),
                        KeyBinds::Explicit(KeyCode::Delete, Command::Del(0)),
                    ],
                ),
            ]),
        }
    }
}
