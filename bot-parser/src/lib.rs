mod error;
mod time;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Marge(Marge),
    Help,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum Marge {
    Add(chrono::NaiveDateTime),
    Cancel,
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
                "m" | "marge" => Ok(Command::Marge(Marge::try_parse_marge(&tokens[2..])?)),
                "h" | "help" => Ok(Command::Help),
                _ => Err(error::Error::NotACommand(input.into())),
            },
            Option::None => Ok(Command::Help),
        }
    }

    fn lexer(input: &str) -> Vec<&str> {
        input.split_whitespace().collect()
    }
}

impl Marge {
    fn try_parse_marge(input: &[&str]) -> error::Result<Marge> {
        let cmd = input.get(0).map(|s| s.to_lowercase());

        match cmd {
            Some(s) => match s.as_str() {
                "c" | "cancel" => Ok(Marge::Cancel),
                "a" | "add" => Ok(Marge::Add(time::parse_time(input[1])?)),
                "h" | "help" => Ok(Marge::Help),
                _ => Ok(Marge::Add(time::parse_time(input[0])?)),
            },
            Option::None => Ok(Marge::Help),
        }
    }
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
            Command::Marge(Marge::Add(
                chrono::NaiveDate::from_ymd_opt(2024, 11, 30)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
            ))
        );
        assert_eq!(
            Command::try_parse("@bot m add 2024-11-30T12:00", "@bot")?,
            Command::Marge(Marge::Add(
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
        assert_eq!(
            Command::try_parse("@bot m c", "@bot")?,
            Command::Marge(Marge::Cancel)
        );
        Ok(())
    }
}
