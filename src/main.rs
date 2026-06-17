use std::{env, fs, path::Path};

use clap::{Arg, ArgMatches, Command};
use color_eyre::Result;

use mc_motd::{config::Config, server::MOTDServer, utils::read_favicon_from_file};

use log::LevelFilter;
use schemars::schema_for;
use simple_logger::SimpleLogger;

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
            Command::new("generateConfigSchema")
                .visible_alias("gcs")
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
                    Arg::new("favicon_path").long("favicon_path").short('f'),
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
        Some(("generateConfigSchema", sub_matches)) => generate_config_schema(sub_matches),
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

fn generate_config_schema(sub_matches: &ArgMatches) {
    let path_arg = sub_matches.get_one::<String>("config_path");
    let path = path_arg.map(|s| s.as_str()).unwrap_or("config.schema.json");
    let final_path = Path::new(path);
    let schema = schema_for!(Config);
    fs::write(final_path, serde_json::to_string_pretty(&schema).unwrap()).unwrap();
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

    if let Some(favicon_path) = sub_matches.get_one::<String>("favicon_path") {
        let path = Path::new(favicon_path.as_str());
        if !path.exists() {
            panic!("Favicon file doesn't exist!");
        }
        config.favicon = read_favicon_from_file(path).ok();
    }

    let server = MOTDServer::new(config);
    server.start().unwrap();
}
