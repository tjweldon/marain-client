use chrono::prelude::*;
use crossterm::event::KeyCode;
use futures::executor::Enter;
use std::{
    collections::{HashMap, VecDeque},
    fmt::{write, Display},
};

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

#[derive(Debug, Clone, Hash)]
pub enum Command {
    Reset,
    Quit,
    Capture(char),
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
            Enter(Mode::Navigate) => "Enter Navigation Mode",
            Enter(Mode::Insert) => "Enter Insert Mode",
            SendBuffer => "Send Message",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug)]
pub struct Log(DateTime<Utc>, String, String);

impl Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} | {} ]: {}",
            self.0.format("%Y-%m-%d %H:%M:%S").to_string(),
            self.1,
            self.2
        )
    }
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub buffer: String,
    pub logs: VecDeque<Log>,
    pub mode: Mode,
    pub keymaps: ModalKeyMaps,
    pub username: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            mode: Mode::Navigate,
            buffer: "".into(),
            logs: VecDeque::new(),
            keymaps: ModalKeyMaps::default(),
            username: format!("User {}", Utc::now().timestamp_micros() % 1024,),
        }
    }

    pub fn map_key(&self, code: KeyCode) -> Option<Command> {
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

    pub fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::Quit => {
                self.should_quit = true;
            }
            Command::Reset => {
                self.buffer = "".into();
            }
            Command::Capture(c) => {
                self.buffer.push(c);
            }
            Command::Enter(mode) => {
                self.mode = mode;
            }
            Command::SendBuffer => {
                self.logs
                    .push_front(Log(Utc::now(), self.username.clone(), self.buffer.clone()));
                if self.logs.len() > 100 {
                    self.logs.pop_back();
                }
                self.buffer = "".into();
            }
        }
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
                        KeyBinds::Explicit(KeyCode::Esc, Command::Enter(Mode::Navigate)),
                        KeyBinds::Explicit(KeyCode::Enter, Command::SendBuffer),
                        KeyBinds::capture(),
                    ],
                ),
            ]),
        }
    }
}
