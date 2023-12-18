use clap::{Parser, Subcommand};
use std::error::Error;

use crate::config::Config;
use crate::rf::RFClient;

pub mod pair;
pub mod sensors;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(
        global = true,
        short,
        long,
        value_name = "FILE",
        default_value = "config.toml"
    )]
    pub config: String,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// pair with boiler
    Pair,
    /// get sensors
    Sensors,
}

pub fn parse() -> Cli {
    Cli::parse()
}

impl Cli {
    pub fn run(
        &self,
        rf: &mut Box<dyn RFClient>,
        config: &mut Config,
    ) -> Result<(), Box<dyn Error>> {
        match self.command {
            Some(Commands::Pair) => pair::run(rf, config),
            Some(Commands::Sensors) => sensors::run(rf, config),
            None => {
                println!("main");
                Ok(())
            }
        }
    }
}
