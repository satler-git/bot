mod error;
mod time;

/// Commands are not case sensitive
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// marge and m
    Marge(Marge),
    /// help and h
    Help,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum Marge {
    NoArgs(chrono::NaiveDateTime),
    /// help and h
    #[default]
    Help,
}

impl Command {
    pub fn try_parse(input: &str, mention: &str) -> error::Result<Command> {
        let tokens = Self::lexer(input);

        if tokens.get(0) != Some(&mention) {
            return Err(error::Error::NotACommand(input.into()));
        }

        let cmd = tokens.get(1).map(|s| s.to_lowercase());

        match cmd {
            Some(s) => match s.as_str() {
                "m" | "marge" => Ok(Command::Marge(Self::try_parse_marge(&tokens[2..])?)),
                "h" | "help" => Ok(Command::Help),
                _ => Err(error::Error::NotACommand(input.into())),
            },
            Option::None => Ok(Command::Help),
        }
    }

    fn try_parse_marge(input: &[&str]) -> error::Result<Marge> {
        if input.is_empty() || input[0].to_lowercase() == "h" || input[0].to_lowercase() == "help" {
            return Ok(Marge::Help);
        }
        Ok(Marge::NoArgs(time::parse_time(input[0])?))
    }

    fn lexer(input: &str) -> Vec<&str> {
        input.split_whitespace().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Command, Marge};

    #[test]
    fn test_parse_simple_help() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(Command::try_parse("@bot h", "@bot")?, Command::Help);
        assert_eq!(Command::try_parse("@bot HELP", "@bot")?, Command::Help);
        assert_eq!(Command::try_parse("@bot help", "@bot")?, Command::Help);
        assert_eq!(Command::try_parse("@bot H", "@bot")?, Command::Help);
        assert_eq!(Command::try_parse("@bot h a a", "@bot")?, Command::Help);
        assert_eq!(Command::try_parse("@bot", "@bot")?, Command::Help);
        Ok(())
    }

    #[test]
    fn test_parse_parse_marge() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Command::try_parse("@bot m 2024-11-30T12:00", "@bot")?,
            Command::Marge(Marge::NoArgs(
                chrono::NaiveDate::from_ymd_opt(2024, 11, 30)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
            ))
        );
        assert_eq!(
            Command::try_parse("@bot m h", "@bot")?,
            Command::Marge(Marge::Help)
        );
        Command::try_parse("@bot M 12:00", "@bot")?;
        Ok(())
    }
}
