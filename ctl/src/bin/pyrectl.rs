use std::collections::HashSet;

use clap::{arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use g560_driver::{Driver, Target};
use pyreworks_common::Color;

#[tokio::main]
async fn main() {
    let matches = clap::Command::new(env!("CARGO_CRATE_NAME"))
        .about("Ambience light control program")
        .subcommand(
            Command::new("g560")
                .about("Control Logitech G560 speaker lights")
                .subcommand(
                    Command::new("set")
                        .about("Set mode and/or state for speaker lights")
                            .subcommand(
                                Command::new("off")
                                    .about("Set speaker lights off")
                                    .arg(arg!(-t --target <TARGET> "the light target, e.g. \"left-primary\", \"right-secondary\", or \"all\"")
                                        .action(ArgAction::Append)
                                        .default_value("all")))
                            .subcommand(
                                Command::new("solid")
                                    .about("Set speaker lights to solid color")
                                    .arg(arg!(-c --color <COLOR> "the color to set, e.g. \"#FFFFFF\" or \"white\"")
                                        .required(true))
                                    .arg(arg!(-t --target <TARGET> "the light target, e.g. \"left-primary\", \"right-secondary\", or \"all\"")
                                        .action(ArgAction::Append)
                                        .default_value("all")))
                            .subcommand(
                                Command::new("breathe")
                                    .about("Set speaker lights to a breathing color")
                                    .arg(arg!(-c --color <COLOR> "the color to set, e.g. \"#FFFFFF\" or \"white\"")
                                        .required(true))
                                    .arg(arg!(-r --rate <RATE> "the breathing rate to set in milliseconds")
                                        .default_value("10000"))
                                    .arg(arg!(-b --brightness <BRIGHTNESS> "the brightness to breath to, 1-100")
                                        .default_value("100"))
                                    .arg(arg!(-t --target <TARGET> "the light target, e.g. \"left-primary\", \"right-secondary\", or \"all\"")
                                        .action(ArgAction::Append)
                                        .default_value("all")))
                            .subcommand(
                                Command::new("cycle")
                                    .about("Set speaker lights to cycling colors")
                                    .arg(arg!(-r --rate <RATE> "the breathing rate to set in milliseconds")
                                        .default_value("10000"))
                                    .arg(arg!(-b --brightness <BRIGHTNESS> "the brightness to breath to, 1-100")
                                        .default_value("100"))
                                    .arg(arg!(-t --target <TARGET> "the light target, e.g. \"left-primary\", \"right-secondary\", or \"all\"")
                                        .action(ArgAction::Append)
                                        .default_value("all")))))
        .get_matches();

    if let Err(e) = run_cli(&matches).await {
        eprintln!("{}{} {}", "error".bold().red(), ":".bold(), e.to_string());
    }
}

async fn run_cli(matches: &ArgMatches) -> anyhow::Result<()> {
    if let Some(matches) = matches.subcommand_matches("g560") {
        if let Some(matches) = matches.subcommand_matches("set") {
            if let Some(matches) = matches.subcommand_matches("off") {
                let targets = matches.get_many::<String>("target")
                    .map(|v| v.into_iter().collect::<Vec<_>>()).unwrap_or(vec![&"all".to_owned()]).into_iter()
                        .flat_map(|t| Target::lookup(t))
                        .collect::<HashSet<_>>(); // Collect to HashSet to dedup targets.

                let commands = targets.into_iter()
                    .map(|t| g560_driver::Command::new_color_off(t))
                    .collect::<Vec<_>>();

                let driver = Driver::connect()?;
                driver.run(&commands).await?;
            }

            if let Some(matches) = matches.subcommand_matches("solid") {
                let targets = matches.get_many::<String>("target").unwrap().into_iter()
                    .flat_map(|t| Target::lookup(t))
                    .collect::<HashSet<_>>();

                let color: Color = matches.get_one::<String>("color").unwrap().parse()?;

                let commands = targets.into_iter()
                    .map(|t| g560_driver::Command::new_color_solid(t, color.clone()))
                    .collect::<Vec<_>>();

                let driver = Driver::connect()?;
                driver.run(&commands).await?;
            }

            if let Some(matches) = matches.subcommand_matches("breathe") {
                let targets = matches.get_many::<String>("target").unwrap().into_iter()
                    .flat_map(|t| Target::lookup(t))
                    .collect::<HashSet<_>>();

                let color: Color = matches.get_one::<String>("color").unwrap().parse()?;
                let rate: u16 = matches.get_one::<String>("rate").unwrap().parse()?;
                let brightness: u8 = matches.get_one::<String>("brightness").unwrap().parse()?;

                let commands = targets.into_iter()
                    .map(|t| g560_driver::Command::new_color_breathe(t, color.clone(), rate, brightness))
                    .collect::<Vec<_>>();

                let driver = Driver::connect()?;
                driver.run(&commands).await?;
            }

            if let Some(matches) = matches.subcommand_matches("cycle") {
                let targets = matches.get_many::<String>("target").unwrap().into_iter()
                    .flat_map(|t| Target::lookup(t))
                    .collect::<HashSet<_>>();

                let rate: u16 = matches.get_one::<String>("rate").unwrap().parse()?;
                let brightness: u8 = matches.get_one::<String>("brightness").unwrap().parse()?;

                let commands = targets.into_iter()
                    .map(|t| g560_driver::Command::new_color_cycle(t, rate, brightness))
                    .collect::<Vec<_>>();

                let driver = Driver::connect()?;
                driver.run(&commands).await?;
            }
        }
    }

    Ok(())
}