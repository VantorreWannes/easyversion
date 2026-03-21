use std::{env, path::PathBuf};

use anyhow::Context;
use clap::{Arg, Command, ValueHint, builder::ValueParser, value_parser};
use directories::ProjectDirs;
use easyversion::{
    APPLICATION, ORGANIZATION, QUALIFIER,
    operations::{Version, clean, history, save, split},
    store::FileStore,
};
use log::{info, warn};

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
                        .value_parser(value_parser!(PathBuf))
                        .help("Destination directory path")
                        .required(true),
                )
                .arg(
                    Arg::new("overwrite")
                        .short('o')
                        .long("overwrite")
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

fn execute(
    matches: clap::ArgMatches,
    data_store: FileStore,
    history_store: FileStore,
    current_directory: PathBuf,
) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("save", sub_matches)) => {
            let comment = sub_matches.get_one::<String>("comment").cloned();
            info!("Running save command");
            save(&data_store, &history_store, &current_directory, comment)
                .context("Failed to save version")?;
            Ok(())
        }
        Some(("list", _)) => {
            info!("Running list command");
            if let Some(hist) = history(&history_store, &current_directory)? {
                for (i, snapshot) in hist.snapshots.iter().enumerate() {
                    let comment = snapshot.comment.as_deref().unwrap_or("No comment");
                    println!("{}: {}", i + 1, comment);
                }
            } else {
                println!("No versions found for this directory.");
            }
            Ok(())
        }
        Some(("split", sub_matches)) => {
            let path = sub_matches.get_one::<PathBuf>("path").unwrap();
            let version_idx = sub_matches.get_one::<usize>("version").copied();

            let version = match version_idx {
                Some(idx) => Version::Specific(idx.saturating_sub(1)),
                None => Version::Latest,
            };

            let overwrite = sub_matches.get_flag("overwrite");

            if path.exists() && !overwrite {
                warn!("Target path already exists. Refusing to overwrite.");
                anyhow::bail!("Target path already exists. Use --overwrite to ignore.");
            }

            info!("Running split command to {:?}", path);
            split(
                &data_store,
                &history_store,
                &current_directory,
                path,
                version,
            )
            .context("Failed to split workspace")?;
            Ok(())
        }
        Some(("clean", _)) => {
            info!("Running clean command");
            clean(&data_store, &history_store, &current_directory)
                .context("Failed to clean workspace")?;
            Ok(())
        }

        _ => unreachable!("Clap should ensure we don't get here"),
    }
}

fn easyversion() -> anyhow::Result<()> {
    let project_directories = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| anyhow::anyhow!("No home directory could be found"))?;

    let data_directory = project_directories.data_local_dir().to_path_buf();

    let data_store = FileStore::new(&data_directory.join("data"))?;
    let history_store = FileStore::new(&data_directory.join("history"))?;

    let current_directory =
        env::current_dir().context("Failed to get current working directory")?;

    let matches = command().get_matches();

    execute(matches, data_store, history_store, current_directory)
}

fn main() {
    env_logger::init();

    if let Err(err) = easyversion() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_command() {
        let dir = tempdir().unwrap();
        let data_store = FileStore::new(&dir.path().join("data")).unwrap();
        let history_store = FileStore::new(&dir.path().join("history")).unwrap();
        let current_dir = dir.path().join("workspace");
        std::fs::create_dir_all(&current_dir).unwrap();

        std::fs::write(current_dir.join("test.txt"), "hello").unwrap();

        let matches = command().get_matches_from(vec!["ev", "save", "-c", "my comment"]);

        execute(
            matches,
            data_store.clone(),
            history_store.clone(),
            current_dir.clone(),
        )
        .unwrap();

        let hist = history(&history_store, &current_dir).unwrap().unwrap();
        assert_eq!(hist.snapshots.len(), 1);
        assert_eq!(hist.snapshots[0].comment.as_deref(), Some("my comment"));
    }

    #[test]
    fn test_split_command() {
        let dir = tempdir().unwrap();
        let data_store = FileStore::new(&dir.path().join("data")).unwrap();
        let history_store = FileStore::new(&dir.path().join("history")).unwrap();
        let current_dir = dir.path().join("workspace");
        std::fs::create_dir_all(&current_dir).unwrap();

        std::fs::write(current_dir.join("test.txt"), "hello").unwrap();

        let save_matches = command().get_matches_from(vec!["ev", "save"]);
        execute(
            save_matches,
            data_store.clone(),
            history_store.clone(),
            current_dir.clone(),
        )
        .unwrap();

        let target_dir = dir.path().join("target");
        let target_dir_str = target_dir.to_str().unwrap();

        let split_matches = command().get_matches_from(vec!["ev", "split", "-p", target_dir_str]);
        execute(
            split_matches,
            data_store.clone(),
            history_store.clone(),
            current_dir.clone(),
        )
        .unwrap();

        assert!(target_dir.join("test.txt").exists());
    }
}
