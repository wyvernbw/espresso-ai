use std::{collections::HashMap, error::Error, io::stdout};

use crossterm::{
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, Borders, List, ListItem},
};

type AppTerm = tui::Terminal<CrosstermBackend<std::io::Stdout>>;

use crate::{espresso, input::InputEvent, select, Journal};

pub enum KeyAction {
    Quit,
    AddBeans,
    None,
}

impl From<InputEvent> for KeyAction {
    fn from(value: InputEvent) -> Self {
        use crossterm::event::KeyCode::Char;
        if let InputEvent::Input(crossterm::event::Event::Key(key)) = value {
            match key.code {
                Char('q') => KeyAction::Quit,
                Char('b') => KeyAction::AddBeans,
                _ => KeyAction::None,
            }
        } else {
            KeyAction::None
        }
    }
}

pub(crate) fn ui(terminal: &mut AppTerm) -> Result<(), Box<dyn Error + 'static>> {
    terminal.draw(|f| {
        let size = f.size();
        let app = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Max(u16::MAX), Constraint::Length(2)].as_ref())
            .split(f.size());
        let main = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)])
            .split(app[0]);
        let block = Block::default()
            .title("Espresso Ai  ")
            .borders(Borders::TOP);
        f.render_widget(block, size);
        let beans = Block::default().title("Beans").borders(Borders::NONE);
        let beans_list = List::new([ListItem::new("askdlfjslfj")]).block(beans);
        f.render_widget(beans_list, main[0]);
        let journal_page = Block::default().title("Journal").borders(Borders::NONE);
        f.render_widget(journal_page, main[1]);
        let keymaps =
            List::new([ListItem::new("q - quit")]).block(Block::default().title("Keymaps"));
        f.render_widget(keymaps, app[1]);
    })?;

    Ok(())
}
