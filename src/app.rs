use std::error::Error;

use crossterm::event::Event;
use tui::widgets::Row;
use tui::widgets::Table;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::journal::Journal;

type AppTerm = tui::Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Debug)]
pub enum KeyAction {
    Quit,
    AddBeans,
    Left,
    Right,
    Up,
    Down,
    None,
    Confirm,
    Type(char),
    Backspace,
    Cancel,
}

impl From<Event> for KeyAction {
    fn from(value: Event) -> Self {
        use crossterm::event::KeyCode;
        use crossterm::event::KeyCode::Char;
        if let crossterm::event::Event::Key(key) = value {
            if key.kind != crossterm::event::KeyEventKind::Press {
                KeyAction::None
            } else {
                match key.code {
                    Char('q') => KeyAction::Quit,
                    Char('b') => KeyAction::AddBeans,
                    Char('h') | KeyCode::Left => KeyAction::Left,
                    Char('l') | KeyCode::Right => KeyAction::Right,
                    Char('j') | KeyCode::Down => KeyAction::Down,
                    Char('k') | KeyCode::Up => KeyAction::Up,
                    KeyCode::Enter => KeyAction::Confirm,
                    KeyCode::Backspace => KeyAction::Backspace,
                    KeyCode::Esc => KeyAction::Cancel,
                    KeyCode::Char(c) => KeyAction::Type(c),
                    _ => KeyAction::None,
                }
            }
        } else {
            KeyAction::None
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub(crate) enum Focus {
    BeansList,
    #[default]
    JournalPage,
}

impl Focus {
    pub fn left(&self) -> Option<Focus> {
        match self {
            Self::BeansList => None,
            Self::JournalPage => Some(Self::BeansList),
        }
    }
    pub fn right(&self) -> Option<Focus> {
        match self {
            Self::BeansList => Some(Self::JournalPage),
            Self::JournalPage => None,
        }
    }
    pub fn get_style(&self, value: Focus, text: String) -> Spans {
        if *self == value {
            Spans(vec![Span::styled(
                text,
                Style::default().fg(tui::style::Color::Magenta),
            )])
        } else {
            Spans(vec![Span::raw(text)])
        }
    }
}

#[derive(Debug, Default)]
pub struct BeansListState(ListState);

impl BeansListState {
    pub fn next(&mut self, journal: &Journal) {
        let BeansListState(list) = self;
        let idx = match list.selected() {
            Some(idx) => idx,
            None if !journal.is_empty() => 0,
            None => return,
        };
        let size = journal.len();
        let idx = idx.saturating_add(1).clamp(0, size - 1);
        list.select(Some(idx));
    }
    pub fn prev(&mut self, journal: &Journal) {
        let BeansListState(list) = self;
        let idx = match list.selected() {
            Some(idx) => idx,
            None if !journal.is_empty() => 0,
            None => return,
        };
        let size = journal.len();
        let idx = idx.saturating_sub(1).clamp(0, size - 1);
        list.select(Some(idx));
    }
    pub fn select_first(&mut self) {
        let BeansListState(list) = self;
        list.select(Some(0));
    }
}

#[derive(Default)]
pub(crate) struct AppState {
    pub beans_list_state: BeansListState,
    pub focused_window: Focus,
    pub add_beans_popup_text: Option<String>,
}

impl AppState {
    fn get_selected_beans(&self, journal: &Journal) -> Option<String> {
        let BeansListState(list) = &self.beans_list_state;
        if let Some(idx) = list.selected() {
            journal.keys().nth(idx).map(|s| s.to_owned())
        } else {
            None
        }
    }
}

pub(crate) fn ui(
    terminal: &mut AppTerm,
    state: &mut AppState,
    journal: &Journal,
) -> Result<(), Box<dyn Error + 'static>> {
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
        let footer = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(app[1]);

        let block = Block::default()
            .title("Espresso Ai  ")
            .borders(Borders::ALL);
        f.render_widget(block, size);

        let beans = Block::default()
            .title(
                state
                    .focused_window
                    .get_style(Focus::BeansList, "Beans".to_owned()),
            )
            .borders(Borders::NONE);
        let journal_beans: Vec<_> = journal
            .keys()
            .map(|bean| ListItem::new(bean.to_owned()))
            .collect();
        let beans_list = List::new(journal_beans).block(beans).highlight_symbol("=> ").highlight_style(
            Style::default()
                .fg(tui::style::Color::Yellow)
                .add_modifier(tui::style::Modifier::BOLD),
        );
        f.render_stateful_widget(beans_list, main[0], &mut state.beans_list_state.0);

        let journal_header = Row::new(vec!["Date", "In", "Out", "Grind", "Time", "Notes"])
            .style(Style::default().fg(tui::style::Color::Cyan))
            .height(1)
            .bottom_margin(1);
        let journal_block = Block::default().title(state.focused_window.get_style(Focus::JournalPage, "Journal".to_owned())).borders(Borders::NONE);
        let journal_table = Table::new(vec![Row::new(vec!["hello", "world"])]).header(journal_header).block(journal_block)
            .widths(&[Constraint::Percentage(10), Constraint::Percentage(10), Constraint::Percentage(10), Constraint::Percentage(10), Constraint::Percentage(10), Constraint::Percentage(50)]);
        f.render_widget(journal_table, main[1]);

        let keymaps =
            Paragraph::new("q - quit   b - add beans   d - delete beans   c - rename beans   e - add espresso shot").block(Block::default().title("Keymaps"));
        f.render_widget(keymaps, footer[0]);

        if let Some(ref text) = state.add_beans_popup_text {
            let block = Block::default()
                .title("Enter beans (origin, roastery, pet name, can be anything you want)")
                .borders(Borders::ALL);
            let area = centered_rect(60, 20, size);
            let typed = Paragraph::new(text.to_owned()).block(block);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(typed, area);
        }
    })?;

    Ok(())
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
