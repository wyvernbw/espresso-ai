use std::{error::Error, time::Duration};

#[derive(Debug)]
pub struct Shot {
    in_weight: f32,
    out_weight: f32,
    grind: String,
    temperature: f32,
    time: Duration,
}

#[derive(Debug)]
pub enum ShotError {
    Weight,
    Temperature,
    Grind,
    Time,
}

impl std::fmt::Display for ShotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ShotError::*;
        let msg = match self {
            Weight => "The weight you chose is not valid!",
            Temperature => "The temperature is invalid!",
            Grind => "Error registering grind setting",
            Time => "The time is invalid!",
        };
        write!(f, "{} 🤡", msg)
    }
}

impl Error for ShotError {}

impl Shot {
    pub fn ask() -> Result<Shot, Box<dyn Error + 'static>> {
        println!("☕ enter shot details:");
        let in_weight = inquire::Text::new("in (grams): ")
            .prompt()
            .map_err(|_| ShotError::Weight)?
            .parse::<f32>()
            .map_err(|_| ShotError::Weight)?;
        let out_weight = inquire::Text::new("out (grams): ")
            .prompt()
            .map_err(|_| ShotError::Weight)?
            .parse::<f32>()
            .map_err(|_| ShotError::Weight)?;
        let grind = inquire::Text::new("grind setting (any format): ")
            .prompt()
            .map_err(|_| ShotError::Grind)?;
        let temperature = inquire::Text::new("temperature (celesius): ")
            .prompt()
            .map_err(|_| ShotError::Temperature)?
            .parse::<f32>()
            .map_err(|_| ShotError::Temperature)?;
        let time = inquire::Text::new("time to extract: ")
            .prompt()
            .map_err(|_| ShotError::Time)?;
        let time = parse_duration::parse(&time).map_err(|_| ShotError::Time)?;
        Ok(Shot {
            in_weight,
            out_weight,
            grind,
            temperature,
            time,
        })
    }
}
