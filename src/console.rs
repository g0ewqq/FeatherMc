use std::io::{self, BufRead};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleCommand {
    Stop,
    Help,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Shutdown,
    Command(ConsoleCommand),
}

#[must_use]
pub fn parse(input: &str) -> Option<ConsoleCommand> {
    let line = input.trim();
    if line.is_empty() {
        return None;
    }
    match line.split_whitespace().next() {
        Some(word) if word.eq_ignore_ascii_case("stop") => Some(ConsoleCommand::Stop),
        Some(word) if word.eq_ignore_ascii_case("help") => Some(ConsoleCommand::Help),
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
    fn keeps_unknown_input() {
        assert_eq!(
            parse("restart now"),
            Some(ConsoleCommand::Unknown("restart now".to_owned()))
        );
    }
}
