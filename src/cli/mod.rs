mod build_map;
mod test;

use std::str::FromStr;

use thiserror::Error;

use structopt::StructOpt;

pub use self::build_map::BuildMapCommand;
pub use self::test::TestCommand;

#[derive(Debug, StructOpt)]
#[structopt(name = "Test", about, author)]
pub struct Options {
    #[structopt(flatten)]
    pub global: GlobalOptions,

    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

impl Options {
    pub fn run(self) -> anyhow::Result<()> {
        match self.subcommand {
            Subcommand::Test(subcommand) => subcommand.run(),
            Subcommand::BuildMap(subcommand) => subcommand.run(),
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct GlobalOptions {
    /// Sets verbosity level. Can be specified multiple times.
    #[structopt(long("verbose"), short, global(true), parse(from_occurrences))]
    pub verbosity: u8,

    /// Set color behavior. Valid values are auto, always, and never.
    #[structopt(long("color"), global(true), default_value("auto"))]
    pub color: ColorChoice,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl FromStr for ColorChoice {
    type Err = ColorChoiceParseError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match source {
            "auto" => Ok(ColorChoice::Auto),
            "always" => Ok(ColorChoice::Always),
            "never" => Ok(ColorChoice::Never),
            _ => Err(ColorChoiceParseError {
                attempted: source.to_owned(),
            }),
        }
    }
}

impl From<ColorChoice> for termcolor::ColorChoice {
    fn from(value: ColorChoice) -> Self {
        match value {
            ColorChoice::Auto => termcolor::ColorChoice::Auto,
            ColorChoice::Always => termcolor::ColorChoice::Always,
            ColorChoice::Never => termcolor::ColorChoice::Never,
        }
    }
}

impl From<ColorChoice> for env_logger::WriteStyle {
    fn from(value: ColorChoice) -> Self {
        match value {
            ColorChoice::Auto => env_logger::WriteStyle::Auto,
            ColorChoice::Always => env_logger::WriteStyle::Always,
            ColorChoice::Never => env_logger::WriteStyle::Never,
        }
    }
}

#[derive(Debug, Error)]
#[error("Invalid color choice '{attempted}'. Valid values are: auto, always, never")]
pub struct ColorChoiceParseError {
    attempted: String,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    Test(TestCommand),
    BuildMap(BuildMapCommand),
}
