pub mod error;
mod time;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Merge(Merge),
    Help,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum Merge {
    Add(chrono::NaiveDateTime),
    Cancel,
    #[default]
    Help,
}

impl Command {
    pub fn try_parse(input: &str, mention: &str) -> error::Result<Command> {
        let tokens = Self::lexer(input);

        if tokens.first() != Some(&mention) {
            return Err(error::Error::NotAMention);
        }

        let cmd = tokens.get(1).map(|s| s.to_lowercase());

        match cmd {
            Some(s) => match s.as_str() {
                "m" | "merge" => Ok(Command::Merge(Merge::try_parse_merge(&tokens[2..])?)),
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

impl Help for Command {
    const HELP: &str = "
You can run commands like `@mention help`.

# Commands

All commands are case-insensitive.
Commands may have shorthand versions; for example, `help` is semantically equivalent to `h`.
If you run a command without arguments, the help message will be displayed.

- `merge` (`m`): View the help for the merge command (`merge help`).
    - This command can only be used on Pull Requests.
- `help` (`h`): Display this help message.
";
}

impl Merge {
    fn try_parse_merge(input: &[&str]) -> error::Result<Merge> {
        let cmd = input.first().map(|s| s.to_lowercase());

        match cmd {
            Some(s) => match s.as_str() {
                "c" | "cancel" => Ok(Merge::Cancel),
                "a" | "add" => Ok(Merge::Add(time::parse_time(input[1])?)),
                "h" | "help" => Ok(Merge::Help),
                _ => Ok(Merge::Add(time::parse_time(input[0])?)),
            },
            Option::None => Ok(Merge::Help),
        }
    }
}

impl Help for Merge {
    const HELP: &str = "
`merge` command help.

# Sub-commands

- `add` (`a`): Schedule automatic merging.
    - You can run this command like this:
        - `merge add 16:00`
            - Schedules merging at 16:00 today.
        - `merge add 2024-12-31T16:00`
            - Schedules merging at 16:00 on 2024-12-31.
- `cancel` (`c`): Cancel a scheduled merge.
- `help` (`h`): Display this help message.

Running the command **without sub-commands** acts as an alias for `add`.
";
}

pub trait Help {
    const HELP: &str;
}

#[cfg(test)]
mod tests {
    use super::{Command, Merge};

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
    fn test_parse_parse_merge() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Command::try_parse("@bot m 2024-11-30T12:00", "@bot")?,
            Command::Merge(Merge::Add(
                chrono::NaiveDate::from_ymd_opt(2024, 11, 30)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
            ))
        );
        assert_eq!(
            Command::try_parse("@bot m add 2024-11-30T12:00", "@bot")?,
            Command::Merge(Merge::Add(
                chrono::NaiveDate::from_ymd_opt(2024, 11, 30)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap()
            ))
        );
        assert_eq!(
            Command::try_parse("@bot m h", "@bot")?,
            Command::Merge(Merge::Help)
        );
        Command::try_parse("@bot M 12:00", "@bot")?;
        assert_eq!(
            Command::try_parse("@bot m c", "@bot")?,
            Command::Merge(Merge::Cancel)
        );
        Ok(())
    }
}
