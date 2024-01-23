use std::error::Error;

pub mod cmd;
pub mod config;
pub mod connect;
pub mod datasource;
pub mod rf;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = cmd::parse();

    println!("frisquet-connect");

    // read config
    let mut config = config::read(&cli.config)?;
    // setup rf
    let mut client = rf::new(&config)?;

    cli.run(&mut client, &mut config)?;
    return Ok(config.write()?);
}
