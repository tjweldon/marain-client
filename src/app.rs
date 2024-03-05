use chrono::prelude::*;
use crossterm::event::KeyCode;
use log2 as log;
use marain_api::prelude::{ClientMsgBody, Key};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    chat_log::{Log, LogStyle},
    default_keybinds,
    tui_framework::Event,
    user_config::UserConfig,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Mode {
    Navigate,
    Insert,
    InsertCommand,
    Disconnected,
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
    GetServerTime,
    MoveRooms(Option<String>),
    SendStagedCommand,
    AbortStagedCommand,
    ToggleDebug,
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
            Enter(Mode::InsertCommand) => "Enter command params mode",
            Enter(Mode::Disconnected) => "Disconnect from server",
            SendBuffer => "Send Message",
            GetServerTime => "Get Server Time",
            MoveRooms(..) => "Move rooms",
            SendStagedCommand => "Send Staged Command",
            AbortStagedCommand => "Abort Command Staging",
            ToggleDebug => "Toggle debug output",
        };
        write!(f, "{s}")
    }
}

impl Command {
    fn parse_params(&self, params: String) -> Option<Self> {
        match self {
            Command::MoveRooms(None) => Some(Command::MoveRooms(Some(params))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub show_debug: bool,
    pub buffer: Vec<String>,
    pub caret_offset: (usize, usize),
    pub logs: VecDeque<Log>,
    pub mode: Mode,
    pub staged_command: Option<Command>,
    pub keymaps: ModalKeyMaps,
    pub username: String,
    pub token: Option<String>,
    shared_secret: Option<Key>,
    pub command_sink: Option<UnboundedSender<Event>>,
}

impl App {
    pub fn new(config: UserConfig) -> Self {
        Self {
            should_quit: false,
            show_debug: false,
            buffer: vec!["".into()],
            caret_offset: (1, 1),
            logs: VecDeque::new(),
            mode: Mode::Navigate,
            staged_command: None,
            keymaps: ModalKeyMaps::default(),
            username: config.get_username(),
            token: None,
            shared_secret: None,
            command_sink: None,
        }
    }

    pub fn set_shared_secret(&mut self, shared_secret: Key) {
        self.shared_secret = Some(shared_secret);
    }

    pub fn set_send_chan(&mut self, chan: UnboundedSender<Event>) {
        self.command_sink = Some(chan);
    }

    pub fn map_key(&self, code: KeyCode) -> Option<Command> {
        log::info!("App mapping key {code:?}");
        self.keymaps.get_cmd(&self.mode, code)
    }

    pub fn render_logs(&self, max_messages: usize, log_style: &LogStyle) -> Text {
        self.logs
            .iter()
            .filter(|l| l.should_render(self.show_debug))
            .collect::<Vec<_>>()
            .iter()
            .take(max_messages)
            .rev()
            .map(|l| l.render(log_style))
            .collect::<Vec<Line>>()
            .into()
    }

    pub fn show_current_mode(&self) -> String {
        format!("{}", self.mode)
    }

    pub fn render_keymap(&self) -> Text {
        self.keymaps.render(&self.mode)
    }

    pub fn get_caret_2d(&self) -> (usize, usize) {
        self.caret_offset
    }

    pub fn set_caret_2d(&mut self, row: usize, col: usize) {
        self.caret_offset.0 = row.clamp(1, self.buffer.len());
        self.caret_offset.1 = col.clamp(
            1,
            self.buffer[row.checked_sub(1).unwrap_or(0) % self.buffer.len()].len() + 1,
        );
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
            .fold("".to_string(), |acc, el| acc + &el);
        line_vec.push(Span::raw(subsequent_lines));

        Line::from(line_vec)
    }

    pub fn handle_toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
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
            Command::GetServerTime => self.send_server_command(cmd),
            Command::ToggleDebug => self.handle_toggle_debug(),

            // Any commands requiring user input should go here
            Command::MoveRooms(None) => {
                self.stage_command(cmd);
                self.switch_mode(Mode::InsertCommand);
            }

            // this arm handles sending any parametrised commands
            Command::SendStagedCommand => {
                self.handle_send_staged_command();
            }
            Command::AbortStagedCommand => {
                self.handle_abort_staged_command();
            }

            // ignored patterns
            Command::MoveRooms(Some(_)) => {}
        };
        log::info!("Caret: {:?}", self.caret_offset);
    }

    fn send_server_command(&self, cmd: Command) {
        let body = match cmd {
            Command::GetServerTime => ClientMsgBody::GetTime,
            Command::MoveRooms(Some(target)) => ClientMsgBody::Move { target },
            _ => todo!(),
        };
        if let (Some(ref chan), Some(tok)) = (self.command_sink.clone(), self.token.clone()) {
            match chan.send(Event::ServerCommand {
                token: tok.clone(),
                username: self.username.clone(),
                timestamp: Utc::now(),
                message_body: body,
            }) {
                Err(e) => {
                    log::error!("Failed to send server: {e}");
                }
                _ => {}
            };
        }
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

    fn stage_command(&mut self, command: Command) {
        self.staged_command = Some(command);
    }

    pub fn input_area_name(&self) -> String {
        match (self.mode.clone(), self.staged_command.clone()) {
            (Mode::InsertCommand, Some(command)) => {
                format!("CMD: {command}")
            }
            _ => "MSG".to_string(),
        }
    }

    fn handle_send_staged_command(&mut self) {
        let Some(cmd) = self.staged_command.clone() else {
            log::error!("Called handler for sending staged command with no staged command");
            return;
        };
        let param_string = self.render_buf();
        if let Some(command_with_params) = cmd.parse_params(param_string) {
            self.send_server_command(command_with_params);
        }
        self.buffer = vec!["".into()];
        self.caret_offset = (1, 1);
        self.staged_command = None;
        self.switch_mode(Mode::Navigate);
    }

    fn handle_abort_staged_command(&mut self) {
        if let Some(_) = self.staged_command.clone() {
            self.staged_command = None;
        }

        self.buffer = vec!["".into()];
        self.caret_offset = (1, 1);
        self.switch_mode(Mode::Navigate);
    }

    pub fn switch_mode(&mut self, mode: Mode) {
        self.mode = mode;
        match self.mode {
            Mode::Insert => {
                let (r, c) = self.get_caret_2d();
                self.set_caret_2d(r, c);
            }
            Mode::Navigate => {}
            Mode::InsertCommand => {}
            Mode::Disconnected => {}
        }
    }

    pub fn handle_send(&mut self) {
        let chat_log = Log::new(self.username.clone(), self.render_buf());
        if let (Some(ref chan), Some(tok)) = (self.command_sink.clone(), self.token.clone()) {
            let Ok(_) = chan.send(Event::Send {
                token: tok.clone(),
                username: self.username.clone(),
                timestamp: chat_log.get_ts(),
                contents: chat_log.get_msg_body(),
            }) else {
                return;
            };
        }
        self.buffer = vec!["".into()];
        self.caret_offset = (1, 1);
    }

    pub fn push_debug_log(&mut self, data: impl Debug) {
        self.logs.push_front(Log::new_debug(data));
    }

    fn log_count(&self) -> usize {
        self.logs
            .iter()
            .filter(|l| l.should_render(self.show_debug))
            .count()
    }

    pub fn push_log(&mut self, log: Log) {
        self.logs.push_front(log);
        if self.log_count() > 100 {
            self.logs.pop_back();
        }
    }

    pub fn replace_logs(&mut self, chat_logs: Vec<Log>) {
        self.logs = VecDeque::new();
        for log in chat_logs {
            self.push_log(log);
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
    pub fn render(&self) -> Option<Line> {
        let s = LogStyle::default();
        let styles = [s.time(), s.uname(), s.msg()];
        let formatted = format!("{}", self);
        if formatted.len() == 0 {
            return None;
        }

        let splits = formatted.splitn(3, " ");

        Some(
            Line::default().spans(
                splits
                    .zip(styles)
                    .map(|(chunk, style)| {
                        Span::styled(
                            format!("{:<width$}", chunk.to_string(), width = 5),
                            style.clone(),
                        )
                    })
                    .collect::<Vec<Span>>(),
            ),
        )
    }

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

    fn render(&self, mode: &Mode) -> Text {
        if let Some(binds) = self.keymaps.get(mode) {
            binds
                .iter()
                .filter_map(KeyBinds::render)
                .collect::<Vec<Line>>()
                .into()
        } else {
            Text::default()
        }
    }
}

impl Default for ModalKeyMaps {
    fn default() -> Self {
        Self {
            keymaps: HashMap::from(default_keybinds::keys()),
        }
    }
}
