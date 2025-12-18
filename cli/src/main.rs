use anyhow::Context;
use clap::{Arg, Command, ValueHint, builder::ValueParser, value_parser};
use directories::ProjectDirs;
use easyversion::easyversion::EasyVersion;
use easyversion::logging::EvLogger;
use easyversion::{APPLICATION, ORGANIZATION, QUALIFIER};
use std::env;
use std::path::PathBuf;

fn command() -> Command {
    Command::new("ev")
        .about("Easy Version Control System. Designed for Artists, Musicians, and Game Developers")
        .subcommand_required(true)
        .subcommand(
            Command::new("save")
                .about("Save current state of a folder")
                .arg(
                    Arg::new("comment")
                        .short('c')
                        .long("comment")
                        .value_name("COMMENT")
                        .value_hint(ValueHint::Other)
                        .value_parser(ValueParser::string())
                        .help("Optional comment")
                        .required(false),
                ),
        )
        .subcommand(Command::new("list").about("List saved versions"))
        .subcommand(
            Command::new("split")
                .about("Create a new folder with the project state at a version")
                .arg(
                    Arg::new("path")
                        .short('p')
                        .long("path")
                        .value_name("PATH")
                        .value_hint(ValueHint::DirPath)
                        .value_parser(ValueParser::path_buf())
                        .help("Destination directory path")
                        .required(true),
                )
                .arg(
                    Arg::new("overwrite")
                        .short('o')
                        .long("o")
                        .action(clap::ArgAction::SetTrue)
                        .help("Allow overwriting the target directory")
                        .required(false),
                )
                .arg(
                    Arg::new("version")
                        .short('v')
                        .long("version")
                        .value_name("VERSION")
                        .value_hint(ValueHint::Other)
                        .value_parser(value_parser!(usize))
                        .help("Version index (1..N). Defaults to latest")
                        .required(false),
                ),
        )
        .subcommand(Command::new("clean").about("Cleanup EV in this folder"))
}

fn ev() -> anyhow::Result<()> {
    let project_directories = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| anyhow::anyhow!("No home directory could be found"))?;

    EvLogger::init(&project_directories).context("Failed to initalise EV logger")?;

    let config_directory = project_directories.config_local_dir().to_path_buf();
    let data_directory = project_directories.data_local_dir().to_path_buf();
    let current_directory =
        env::current_dir().context("Failed to get current working directory")?;

    let matches = command().get_matches();
    let easy_version = EasyVersion::new(&config_directory, &data_directory);

    match matches.subcommand() {
        Some(("save", sub_matches)) => {
            let comment = sub_matches.get_one::<String>("comment").map(|s| s.as_str());
            easy_version
                .save(&current_directory, comment)
                .context("Failed to save version")?;
            Ok(())
        }
        Some(("list", _)) => {
            easy_version
                .list(&current_directory)
                .context("Failed to list versions")?;
            Ok(())
        }
        Some(("split", sub_matches)) => {
            let path = sub_matches.get_one::<PathBuf>("path").unwrap();
            let version = sub_matches.get_one::<usize>("version").copied();
            let overwrite = sub_matches.get_flag("overwrite");
            easy_version
                .split(&current_directory, path, version, overwrite)
                .context("Failed to split workspace")?;
            Ok(())
        }
        Some(("clean", _)) => {
            easy_version
                .clean(&current_directory)
                .context("Failed to clean workspace")?;
            Ok(())
        }
        _ => unreachable!("Clap should ensure we don't get here"),
    }
}

fn main() {
    if let Err(err) = ev() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}
