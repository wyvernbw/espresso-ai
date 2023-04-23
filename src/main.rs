mod app;
mod espresso;
mod input;
mod journal;

use app::KeyAction;
use input::Events;
use std::{
    collections::HashMap,
    error::Error,
    io::{self, stdout},
    time::Duration,
};
use tui::backend::CrosstermBackend;

use tui::Terminal;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event::Key, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
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

    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);
    loop {
        app::ui(&mut terminal).expect("failed to draw ui");
        let Ok(event) = events.next() else { break; };
        let event = KeyAction::from(event);
        match event {
            KeyAction::Quit => {
                break;
            }
            KeyAction::AddBeans => {}
            _ => {}
        }
        //let action = select("Select an option: ", Action::get_list())?;
        //if let Action::Exit = action {
        //    break;
        //}

        //let journal: HashMap<String, Vec<espresso::Shot>> = HashMap::new();
        //let journal = run_action(action, journal)?;

        //let mut convo: Vec<Message> = vec![];
        //let body = send_message(
        //    &mut convo,
        //    Message::user("Aki dies in chainsaw man get spoiled lol".to_owned()),
        //)?;
        //dbg!(body);
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
