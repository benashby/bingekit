//! The two screens: picking one item from a list, and putting a list in
//! order. Each screen's key handling is plain state, kept apart from drawing
//! so it can be tested without a terminal.

use std::io;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::{TerminalOptions, Viewport};

/// A key, after mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// `k` or the up arrow.
    Up,
    /// `j` or the down arrow.
    Down,
    /// `K`: move the item under the cursor up.
    MoveUp,
    /// `J`: move the item under the cursor down.
    MoveDown,
    /// Enter.
    Enter,
    /// `q`, Esc or Ctrl+C.
    Cancel,
    /// `1` to `9`.
    Number(usize),
    /// Anything else.
    Other,
}

impl From<KeyEvent> for Key {
    fn from(key: KeyEvent) -> Self {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Self::Cancel,
            KeyCode::Up | KeyCode::Char('k') => Self::Up,
            KeyCode::Down | KeyCode::Char('j') => Self::Down,
            KeyCode::Char('K') => Self::MoveUp,
            KeyCode::Char('J') => Self::MoveDown,
            KeyCode::Enter => Self::Enter,
            KeyCode::Esc | KeyCode::Char('q') => Self::Cancel,
            KeyCode::Char(c @ '1'..='9') => Self::Number(c as usize - '1' as usize),
            _ => Self::Other,
        }
    }
}

/// What a screen does after a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome<T> {
    /// Keep going.
    Pending,
    /// The user confirmed this result.
    Done(T),
    /// The user backed out.
    Cancelled,
}

/// Picking one item from a list.
#[derive(Debug, Clone)]
pub struct Pick {
    items: Vec<String>,
    cursor: usize,
}

impl Pick {
    /// A picker over `items`, with the cursor on the first one.
    #[must_use]
    pub fn new(items: Vec<String>) -> Self {
        Self { items, cursor: 0 }
    }

    /// Handles one key. A number picks that item straight away.
    pub fn key(&mut self, key: Key) -> Outcome<usize> {
        match key {
            Key::Up => self.cursor = self.cursor.saturating_sub(1),
            Key::Down if self.cursor + 1 < self.items.len() => self.cursor += 1,
            Key::Enter => return Outcome::Done(self.cursor),
            Key::Cancel => return Outcome::Cancelled,
            Key::Number(n) if n < self.items.len() => return Outcome::Done(n),
            _ => {}
        }
        Outcome::Pending
    }

    fn lines(&self, prompt: &str) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(prompt.to_owned()), Line::from("")];
        for (i, item) in self.items.iter().enumerate() {
            let marker = if i == self.cursor { ">" } else { " " };
            lines.push(Line::from(format!("{marker} {}. {item}", i + 1)));
        }
        lines
    }
}

/// Putting a list in order.
#[derive(Debug, Clone)]
pub struct Reorder {
    items: Vec<String>,
    order: Vec<usize>,
    cursor: usize,
}

impl Reorder {
    /// A list in its given order, with the cursor on the first item.
    #[must_use]
    pub fn new(items: Vec<String>) -> Self {
        let order = (0..items.len()).collect();
        Self {
            items,
            order,
            cursor: 0,
        }
    }

    /// Handles one key. The result is the new order, as indices into the
    /// original list.
    pub fn key(&mut self, key: Key) -> Outcome<Vec<usize>> {
        let last = self.order.len().saturating_sub(1);
        match key {
            Key::Up => self.cursor = self.cursor.saturating_sub(1),
            Key::Down if self.cursor < last => self.cursor += 1,
            Key::MoveUp if self.cursor > 0 => {
                self.order.swap(self.cursor, self.cursor - 1);
                self.cursor -= 1;
            }
            Key::MoveDown if self.cursor < last => {
                self.order.swap(self.cursor, self.cursor + 1);
                self.cursor += 1;
            }
            Key::Enter => return Outcome::Done(self.order.clone()),
            Key::Cancel => return Outcome::Cancelled,
            _ => {}
        }
        Outcome::Pending
    }

    fn lines(&self, prompt: &str) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(prompt.to_owned()), Line::from("")];
        for (i, &item) in self.order.iter().enumerate() {
            let name = &self.items[item];
            if i == self.cursor {
                lines.push(Line::from(format!("> {}. {name} <", i + 1)));
            } else {
                lines.push(Line::from(format!("  {}. {name}", i + 1)));
            }
        }
        lines
    }
}

/// A screen `run` can drive: it takes keys and draws itself.
trait Screen {
    type Output;
    fn key(&mut self, key: Key) -> Outcome<Self::Output>;
    fn lines(&self, prompt: &str) -> Vec<Line<'static>>;
    fn rows(&self) -> usize;
}

impl Screen for Pick {
    type Output = usize;
    fn key(&mut self, key: Key) -> Outcome<usize> {
        Pick::key(self, key)
    }
    fn lines(&self, prompt: &str) -> Vec<Line<'static>> {
        Pick::lines(self, prompt)
    }
    fn rows(&self) -> usize {
        self.items.len()
    }
}

impl Screen for Reorder {
    type Output = Vec<usize>;
    fn key(&mut self, key: Key) -> Outcome<Vec<usize>> {
        Reorder::key(self, key)
    }
    fn lines(&self, prompt: &str) -> Vec<Line<'static>> {
        Reorder::lines(self, prompt)
    }
    fn rows(&self) -> usize {
        self.items.len()
    }
}

/// Shows a picker in the terminal and returns the chosen index, or `None`
/// when the user backs out.
///
/// # Errors
///
/// Fails when the terminal cannot be used.
pub fn pick(prompt: &str, items: Vec<String>) -> io::Result<Option<usize>> {
    run(&mut Pick::new(items), prompt)
}

/// Shows a reorder screen in the terminal and returns the new order, or
/// `None` when the user backs out.
///
/// # Errors
///
/// Fails when the terminal cannot be used.
pub fn reorder(prompt: &str, items: Vec<String>) -> io::Result<Option<Vec<usize>>> {
    run(&mut Reorder::new(items), prompt)
}

/// Draws a screen inline, below the text already on the terminal, and feeds
/// it keys until it finishes. The terminal is restored on every path out.
fn run<S: Screen>(screen: &mut S, prompt: &str) -> io::Result<Option<S::Output>> {
    let height = u16::try_from(screen.rows() + 2).unwrap_or(u16::MAX);
    let mut terminal = ratatui::try_init_with_options(TerminalOptions {
        viewport: Viewport::Inline(height),
    })?;
    let result = loop {
        let lines = screen.lines(prompt);
        if let Err(e) = terminal.draw(|frame| {
            frame.render_widget(Paragraph::new(lines), frame.area());
        }) {
            break Err(e);
        }
        let key = match event::read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => Key::from(k),
            Ok(_) => continue,
            Err(e) => break Err(e),
        };
        match screen.key(key) {
            Outcome::Pending => {}
            Outcome::Done(value) => break Ok(Some(value)),
            Outcome::Cancelled => break Ok(None),
        }
    };
    ratatui::restore();
    println!();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn pick_moves_and_confirms() {
        let mut pick = Pick::new(names(&["a", "b", "c"]));
        assert_eq!(pick.key(Key::Up), Outcome::Pending);
        assert_eq!(pick.key(Key::Down), Outcome::Pending);
        assert_eq!(pick.key(Key::Down), Outcome::Pending);
        assert_eq!(pick.key(Key::Down), Outcome::Pending);
        assert_eq!(pick.key(Key::Enter), Outcome::Done(2));
    }

    #[test]
    fn pick_takes_numbers_and_ignores_ones_out_of_range() {
        let mut pick = Pick::new(names(&["a", "b"]));
        assert_eq!(pick.key(Key::Number(5)), Outcome::Pending);
        assert_eq!(pick.key(Key::Number(1)), Outcome::Done(1));
        assert_eq!(
            Pick::new(names(&["a"])).key(Key::Cancel),
            Outcome::Cancelled
        );
    }

    #[test]
    fn reorder_moves_items_and_keeps_the_cursor_on_them() {
        let mut list = Reorder::new(names(&["a", "b", "c"]));
        list.key(Key::MoveDown);
        list.key(Key::MoveDown);
        list.key(Key::MoveDown);
        assert_eq!(list.key(Key::Enter), Outcome::Done(vec![1, 2, 0]));
        let mut list = Reorder::new(names(&["a", "b", "c"]));
        list.key(Key::Down);
        list.key(Key::Down);
        list.key(Key::MoveUp);
        assert_eq!(list.key(Key::Enter), Outcome::Done(vec![0, 2, 1]));
    }

    #[test]
    fn reorder_marks_the_cursor_row() {
        let list = Reorder::new(names(&["first", "second"]));
        let text: Vec<String> = list
            .lines("Order:")
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(text, ["Order:", "", "> 1. first <", "  2. second"]);
    }

    #[test]
    fn keys_map_like_the_go_version() {
        let key = |code| Key::from(KeyEvent::from(code));
        assert_eq!(key(KeyCode::Char('j')), Key::Down);
        assert_eq!(key(KeyCode::Char('K')), Key::MoveUp);
        assert_eq!(key(KeyCode::Char('3')), Key::Number(2));
        assert_eq!(key(KeyCode::Esc), Key::Cancel);
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(Key::from(ctrl_c), Key::Cancel);
    }
}
