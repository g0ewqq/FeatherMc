use std::io::{self, BufRead};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

use crate::java::player::GameMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleCommand {
    Stop,
    Help,
    /// Switch every connected player to the given game mode.
    GameMode(GameMode),
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Shutdown,
    Command(ConsoleCommand),
}

pub(crate) fn parse_gamemode(word: &str) -> Option<GameMode> {
    match word.to_ascii_lowercase().as_str() {
        "survival" | "s" | "0" => Some(GameMode::Survival),
        "creative" | "c" | "1" => Some(GameMode::Creative),
        "adventure" | "a" | "2" => Some(GameMode::Adventure),
        "spectator" | "sp" | "3" => Some(GameMode::Spectator),
        _ => None,
    }
}

#[must_use]
pub fn parse(input: &str) -> Option<ConsoleCommand> {
    let line = input.trim();
    if line.is_empty() {
        return None;
    }
    let mut words = line.split_whitespace();
    match words.next() {
        Some(word) if word.eq_ignore_ascii_case("stop") => Some(ConsoleCommand::Stop),
        Some(word) if word.eq_ignore_ascii_case("help") => Some(ConsoleCommand::Help),
        Some(word) if word.eq_ignore_ascii_case("gamemode") => match words.next() {
            Some(arg) => match parse_gamemode(arg) {
                Some(mode) => Some(ConsoleCommand::GameMode(mode)),
                None => Some(ConsoleCommand::Unknown(line.to_owned())),
            },
            None => Some(ConsoleCommand::Unknown(line.to_owned())),
        },
        _ => Some(ConsoleCommand::Unknown(line.to_owned())),
    }
}

pub fn spawn(sender: &Sender<Event>) -> io::Result<JoinHandle<()>> {
    let sender = sender.clone();
    thread::Builder::new()
        .name("feathermc-console".into())
        .spawn(move || {
            for line in io::stdin().lock().lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(_) => break,
                };
                if let Some(command) = parse(&line) {
                    if sender.send(Event::Command(command)).is_err() {
                        break;
                    }
                }
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stop() {
        assert_eq!(parse("stop"), Some(ConsoleCommand::Stop));
        assert_eq!(parse("  STOP  "), Some(ConsoleCommand::Stop));
    }

    #[test]
    fn parses_help() {
        assert_eq!(parse("help"), Some(ConsoleCommand::Help));
        assert_eq!(parse("Help"), Some(ConsoleCommand::Help));
    }

    #[test]
    fn ignores_blank_lines() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("   "), None);
    }

    #[test]
    fn parses_gamemode() {
        assert_eq!(
            parse("gamemode creative"),
            Some(ConsoleCommand::GameMode(GameMode::Creative))
        );
        assert_eq!(
            parse("gamemode 0"),
            Some(ConsoleCommand::GameMode(GameMode::Survival))
        );
        assert_eq!(
            parse("GAMEMODE SP"),
            Some(ConsoleCommand::GameMode(GameMode::Spectator))
        );
        assert_eq!(
            parse("gamemode nope"),
            Some(ConsoleCommand::Unknown("gamemode nope".to_owned()))
        );
        assert_eq!(
            parse("gamemode"),
            Some(ConsoleCommand::Unknown("gamemode".to_owned()))
        );
    }

    #[test]
    fn keeps_unknown_input() {
        assert_eq!(
            parse("restart now"),
            Some(ConsoleCommand::Unknown("restart now".to_owned()))
        );
    }
}
