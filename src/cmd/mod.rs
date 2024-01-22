use clap::{Parser, Subcommand, ValueEnum};
use std::error::Error;

use crate::config::Config;
use crate::rf::RFClient;

pub mod area1;
pub mod data1;
pub mod data2;
pub mod data3;
pub mod data4;
pub mod date;
pub mod pair;
pub mod promiscuous;
pub mod sensors;
pub mod sonde;

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
    Pair {
        #[arg(value_enum)]
        from: Entity,
    },
    /// get sensors
    Sensors,
    /// get data1 - not decoded
    Data1,
    /// get data2 - not decoded
    Data2,
    /// get data3 - not decoded
    Data3,
    /// get data4 - not decoded
    Data4,
    /// get date
    Date,
    /// dump connect messages
    Promiscuous,
    /// set area1 prog
    Area1,
    Sonde,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Entity {
    Connect,
    Satellite_Z1,
    Satellite_Z2,
    Satellite_Z3,
    Sonde,
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
            Some(Commands::Pair { from }) => pair::run(rf, from, config),
            Some(Commands::Sensors) => sensors::run(rf, config),
            Some(Commands::Date) => date::run(rf, config),
            Some(Commands::Promiscuous) => promiscuous::run(rf, config),
            Some(Commands::Area1) => area1::run(rf, config),
            Some(Commands::Data1) => data1::run(rf, config),
            Some(Commands::Data2) => data2::run(rf, config),
            Some(Commands::Data3) => data3::run(rf, config),
            Some(Commands::Data4) => data4::run(rf, config),
            Some(Commands::Sonde) => sonde::run(rf, config),
            None => {
                println!("main");
                Ok(())
            }
        }
    }
}
