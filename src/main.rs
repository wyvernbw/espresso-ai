mod app;
mod espresso;
pub(crate) mod journal;

use app::{AppState, Focus::BeansList, KeyAction};
use journal::Journal;
use std::{
    error::Error,
    io::{self},
    time::Duration,
};
use tui::backend::CrosstermBackend;

use tui::Terminal;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use dotenv::dotenv;
use std::env;

pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";

fn main() -> Result<(), Box<dyn Error + 'static>> {
    dotenv().ok();

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut journal = Journal::new();
    let mut state = AppState::default();
    loop {
        app::ui(&mut terminal, &mut state, &journal).expect("failed to draw ui");

        let Ok(event) = crossterm::event::read() else { break; };
        if let Event::Key(key) = event {
            if let (KeyCode::Char(c), KeyEventKind::Press) = (key.code, key.kind) {
                if let Some(ref mut text) = state.add_beans_popup_text {
                    text.push(c);
                }
            }
        };
        let event = KeyAction::from(event);
        match event {
            KeyAction::Quit if state.add_beans_popup_text.is_none() => {
                break;
            }
            KeyAction::AddBeans if state.add_beans_popup_text.is_none() => {
                state.add_beans_popup_text = Some("".to_owned());
            }
            KeyAction::Left => {
                if let Some(pane) = state.focused_window.left() {
                    state.focused_window = pane;
                }
            }
            KeyAction::Right => {
                if let Some(pane) = state.focused_window.right() {
                    state.focused_window = pane;
                }
            }
            KeyAction::Up => {
                if let BeansList = state.focused_window {
                    state.beans_list_state.prev(&journal);
                }
            }
            KeyAction::Down => {
                if let BeansList = state.focused_window {
                    state.beans_list_state.next(&journal);
                }
            }
            KeyAction::Cancel => {
                if state.add_beans_popup_text.is_some() {
                    state.add_beans_popup_text = None;
                }
            }
            KeyAction::Backspace => {
                if let Some(ref mut text) = state.add_beans_popup_text {
                    text.pop();
                }
            }
            KeyAction::Confirm => match state.add_beans_popup_text {
                Some(ref text) if !text.is_empty() => {
                    let key = text.to_owned();
                    let original_len = journal.len();
                    match journal.get(&key) {
                        None => {
                            journal.insert(text.to_owned(), vec![]);
                            state.add_beans_popup_text = None;
                        }
                        Some(_) => {}
                    }
                    if original_len == 0 {
                        state.beans_list_state.select_first();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    User,
    System,
    Assistant,
}

#[derive(serde::Serialize)]
struct Message {
    role: Role,
    content: String,
}

impl Message {
    fn user(content: String) -> Message {
        Message {
            role: Role::User,
            content,
        }
    }
    fn assistant(content: String) -> Message {
        Message {
            role: Role::Assistant,
            content,
        }
    }
}

#[derive(serde::Serialize)]
struct RequestData<'a> {
    model: &'static str,
    messages: &'a Vec<Message>,
}

impl<'a> RequestData<'a> {
    fn turbo(conversation: &'a Vec<Message>) -> RequestData<'a> {
        RequestData {
            model: "gpt-3.5-turbo",
            messages: conversation,
        }
    }
}

fn send_message(
    conversation: &mut Vec<Message>,
    message: Message,
) -> Result<String, Box<dyn Error + 'static>> {
    let open_ai_key = env::var(OPENAI_API_KEY).expect("expected open ai key in .env");
    conversation.push(message);
    let data = RequestData::turbo(conversation);
    println!("{}", serde_json::to_string(&data)?);
    let body: String = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {open_ai_key}"))
        .send_json(data)?
        .into_string()?;
    Ok(body)
}
