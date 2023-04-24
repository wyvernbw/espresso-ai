use chrono::DateTime;
use std::{error::Error, time::Duration};

#[derive(Debug, Default)]
pub struct Shot {
    pub in_weight: f32,
    pub out_weight: f32,
    pub grind: String,
    pub temperature: f32,
    pub time: Duration,
    pub notes: String,
    pub date: DateTime,
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
