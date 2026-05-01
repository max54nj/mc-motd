use std::{env, path::Path};

use clap::{Arg, ArgMatches, Command, Parser};
use color_eyre::Result;

use mc_motd::{config::Config, server::MOTDServer};

use log::LevelFilter;
use simple_logger::SimpleLogger;

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "The port the server will listen on")]
    port: Option<u16>,
    #[arg(short, long)]
    config_path: Option<String>,
    #[arg(long, help = "", value_name = "<Name>:<UUID>")]
    players: Option<Vec<String>>,
    #[arg(long, help = "")]
    online_players: Option<i32>,
    #[arg(long, help = "")]
    max_players: Option<i32>,
    #[arg(short, long)]
    favicon_path: Option<String>,
}

fn cli() -> Command {
    Command::new("mc-motd")
        .about("A lightweight Minecraft MOTD Mock Server.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("restoreDefaultConfig")
                .visible_alias("rdc")
                .args([Arg::new("config_path").long("config_path").short('c')]),
        )
        .subcommand(
            Command::new("start")
                .about("Start the MOTD Mock Server")
                .args([
                    Arg::new("port")
                        .long("port")
                        .short('p')
                        .help("The port the server will listen on"),
                    Arg::new("config_path").long("config_path").short('c'),
                ]),
        )
}

fn main() -> Result<()> {
    unsafe {
        env::set_var("RUST_BACKTRACE", "full");
    }

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => start_server(sub_matches),
        Some(("restoreDefaultConfig", sub_matches)) => restore_default_config(sub_matches),
        _ => unreachable!(),
    }

    Ok(())
}

fn restore_default_config(sub_matches: &ArgMatches) {
    let path_arg = sub_matches.get_one::<String>("config_path");
    let path = path_arg.map(|s| s.as_str()).unwrap_or("config.json");
    let final_path = Path::new(path);
    let config = Config::new();
    config.write_file(final_path).unwrap();
}

fn start_server(sub_matches: &ArgMatches) {
    let path_arg = sub_matches.get_one::<String>("config_path");
    let path = path_arg.map(|s| s.as_str()).unwrap_or("config.json");
    let final_path = Path::new(path);
    if !final_path.exists() {
        panic!("Config path doesn't exist");
    }
    let mut config = Config::read_file(final_path).unwrap();

    if let Some(port) = sub_matches.get_one::<u16>("port") {
        config.port = *port;
    }

    /* if let Some(players) = args.players {
        let mut parsed_players: Vec<SamplePlayer> = Vec::new();
        for player in players.iter() {
            match player.split_once(':') {
                Some(sp) => parsed_players.push(SamplePlayer {
                    name: sp.0.to_string(),
                    id: sp.1.to_string(),
                }),
                None => log::warn!(
                    "Unable to split \"{}\", check you are using a \":\" to split the name & UUID",
                    player
                ),
            }
        }
        config.players.sample = parsed_players;
    }

    if let Some(online_players) = args.online_players {
        config.players.online = online_players;
    }

    if let Some(max_players) = args.max_players {
        config.players.max = max_players;
    }

    if let Some(favicon_path) = args.favicon_path {
        let path = Path::new(favicon_path.as_str());
        if !path.exists() {
            panic!("Favicon file doesn't exist!");
        }
        config.favicon = read_favicon_from_file(path).ok();
    } */

    let server = MOTDServer::new(config);
    server.start().unwrap();
}
