use crossterm::event::KeyCode;

use crate::app::{CaretMotion, Command, KeyBinds, Mode};

fn disocnnected() -> (Mode, Vec<KeyBinds>) {
    (
        Mode::Disconnected,
        vec![KeyBinds::Explicit(KeyCode::Char('q'), Command::Quit)],
    )
}

fn navigate() -> (Mode, Vec<KeyBinds>) {
    (
        Mode::Navigate,
        vec![
            KeyBinds::Explicit(KeyCode::Char('i'), Command::Enter(Mode::Insert)),
            KeyBinds::Explicit(KeyCode::Char('q'), Command::Quit),
            KeyBinds::Explicit(KeyCode::Char('r'), Command::Reset),
            KeyBinds::Explicit(KeyCode::Char('t'), Command::GetServerTime),
            KeyBinds::Explicit(KeyCode::Char('m'), Command::MoveRooms(None)),
            KeyBinds::Explicit(KeyCode::Char('d'), Command::ToggleDebug),
        ],
    )
}

fn insert() -> (Mode, Vec<KeyBinds>) {
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
    )
}

fn insert_cmd() -> (Mode, Vec<KeyBinds>) {
    (
        Mode::InsertCommand,
        vec![
            // leave insert mode
            KeyBinds::Explicit(KeyCode::Esc, Command::AbortStagedCommand),
            // send message
            KeyBinds::Explicit(KeyCode::Enter, Command::SendStagedCommand),
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
    )
}

pub fn keys() -> [(Mode, Vec<KeyBinds>); 4] {
    [disocnnected(), navigate(), insert(), insert_cmd()]
}
