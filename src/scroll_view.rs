use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyModifiers},
    handle_command,
    style::{ResetColor, SetBackgroundColor},
    terminal::{Clear, ClearType},
    Result,
};

use std::io::Write;

use crate::{
    action::ActionKind,
    tui_util::{move_cursor, AvailableSize, TerminalSize, SELECTED_BG_COLOR},
};

pub struct ScrollView {
    action_kind: ActionKind,
    content: String,
    scroll: usize,
    cursor: Option<usize>,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            action_kind: ActionKind::Quit,
            content: String::with_capacity(1024 * 4),
            scroll: 0,
            cursor: None,
        }
    }
}

impl ScrollView {
    pub fn set_content(&mut self, content: &str, action_kind: ActionKind) {
        self.action_kind = action_kind;
        self.content.clear();
        self.content.push_str(content);
        self.scroll = 0;
        self.cursor = if action_kind.can_select_output() {
            Some(0)
        } else {
            None
        };
    }

    pub fn show<W>(
        &self,
        write: &mut W,
        terminal_size: TerminalSize,
    ) -> Result<()>
    where
        W: Write,
    {
        let line_formatter = self.action_kind.line_formatter::<W>();

        let available_size = AvailableSize::from_temrinal_size(terminal_size);
        handle_command!(write, cursor::MoveTo(0, 1))?;
        for (i, line) in self
            .content
            .lines()
            .skip(self.scroll)
            .take(available_size.height)
            .enumerate()
        {
            if Some(i) == self.cursor {
                handle_command!(write, SetBackgroundColor(SELECTED_BG_COLOR))?;
            }

            handle_command!(write, Clear(ClearType::CurrentLine))?;
            line_formatter(write, line, available_size)?;
            handle_command!(write, cursor::MoveToNextLine(1))?;

            if Some(i) == self.cursor {
                handle_command!(write, ResetColor)?;
            }
        }
        handle_command!(write, Clear(ClearType::FromCursorDown))?;

        Ok(())
    }

    pub fn update<W>(
        &mut self,
        write: &mut W,
        key_event: &KeyEvent,
        terminal_size: TerminalSize,
    ) -> Result<bool>
    where
        W: Write,
    {
        let available_size = AvailableSize::from_temrinal_size(terminal_size);
        match key_event {
            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Down,
                ..
            }
            | KeyEvent {
                code: KeyCode::Enter,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('\n'),
                ..
            } => {
                self.scroll(available_size, 1);
                self.show(write, terminal_size)?;
                Ok(true)
            }
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Up, ..
            } => {
                self.scroll(available_size, -1);
                self.show(write, terminal_size)?;
                Ok(true)
            }
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::PageDown,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char(' '),
                ..
            } => {
                self.scroll(available_size, available_size.height as i32 / 2);
                self.show(write, terminal_size)?;
                Ok(true)
            }
            KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => {
                self.scroll(available_size, available_size.height as i32 / -2);
                self.show(write, terminal_size)?;
                Ok(true)
            }
            KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Home,
                ..
            } => {
                self.scroll = 0;
                if let Some(ref mut cursor) = self.cursor {
                    *cursor = 0;
                }
                self.show(write, terminal_size)?;
                Ok(true)
            }
            KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::End, ..
            } => {
                self.scroll = 0.max(
                    self.content_height(available_size) as i32
                        - available_size.height as i32,
                ) as usize;
                if let Some(ref mut cursor) = self.cursor {
                    *cursor = self.content.lines().count();
                }
                self.show(write, terminal_size)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn content_height(&self, available_size: AvailableSize) -> usize {
        let width = available_size.width;
        self.content
            .lines()
            .map(|l| (l.len() + width - 1) / width)
            .sum()
    }

    fn scroll(&mut self, available_size: AvailableSize, delta: i32) {
        if let Some(ref mut cursor) = self.cursor {
            let line_count = self.content.lines().count();
            move_cursor(
                &mut self.scroll,
                cursor,
                available_size,
                line_count,
                delta,
            );
        } else {
            self.scroll = (self.scroll as i32 + delta)
                .min(
                    self.content_height(available_size) as i32
                        - available_size.height as i32,
                )
                .max(0) as usize;
        }
    }
}
