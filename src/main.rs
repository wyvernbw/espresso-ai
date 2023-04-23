use std::error::Error;

use dotenv::dotenv;
use std::env;

pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";

fn main() -> Result<(), Box<dyn Error + 'static>> {
    dotenv().ok();

    let mut convo: Vec<Message> = vec![];
    let body = send_message(
        &mut convo,
        Message::user("Aki dies in chainsaw man get spoiled lol".to_owned()),
    )?;
    dbg!(body);

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
    body: String,
}

impl Message {
    fn user(body: String) -> Message {
        Message {
            role: Role::User,
            body,
        }
    }
    fn assistant(body: String) -> Message {
        Message {
            role: Role::Assistant,
            body,
        }
    }
}

fn send_message(
    conversation: &mut Vec<Message>,
    message: Message,
) -> Result<String, Box<dyn Error + 'static>> {
    let open_ai_key = env::var(OPENAI_API_KEY).expect("expected open ai key in .env");
    conversation.push(message);
    let body: String = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {open_ai_key}"))
        .send_json(ureq::json!({
            "model": "gpt-3.5-turbo",
            "messages": conversation
        }))?
        .into_string()?;
    Ok(body)
}
