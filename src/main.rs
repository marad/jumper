mod store;
use crate::store::Store;
use anyhow::anyhow;
use clap::{Parser, Subcommand};
use home::home_dir;
use std::env;
use std::fs;

type Res<T> = anyhow::Result<T>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Save { name: Option<String> },
    Get { name: String },
    List,
    Remove { name: String },
    Test,
}

#[tokio::main]
async fn main() -> Res<()> {
    let args = Args::parse();

    let db_path = get_db_path();
    let mut store = Store::create(db_path.as_str()).await?;

    match args.command {
        Commands::Test => {}
        Commands::Save { name } => {
            save_current_path(&mut store, name.as_deref()).await?;
        }
        Commands::Get { name } => {
            get_path(&mut store, &name).await?;
        }
        Commands::List => {
            list_paths(&mut store).await?;
        }
        Commands::Remove { name } => {
            remove_path(&mut store, &name).await?;
        }
    }

    Ok(())
}

fn get_db_path() -> String {
    let home_path = home_dir().expect("Wow, I didn't expect to be homeless!");
    let _ = fs::create_dir_all(format!("{}/.config/jumper", home_path.to_str().unwrap()));
    format!("{}/.config/jumper/db.sqlite", home_path.to_str().unwrap())
}

async fn save_current_path(store: &mut Store, name: Option<&str>) -> Res<()> {
    let current_dir = env::current_dir()?;
    let dir_name = current_dir.file_name().unwrap().to_str().unwrap();
    let save_name = name.unwrap_or(dir_name);
    let save_path = current_dir.to_str().unwrap();
    match store.save(save_name, save_path).await {
        Ok(_) => {}
        Err(err) => match err.downcast_ref::<sqlx::error::Error>() {
            Some(sqlx::error::Error::Database(db_err)) if db_err.is_unique_violation() => {
                println!("Name '{}' is already used for another path.", save_name);
            }
            Some(_) | None => {
                return Err(err);
            }
        },
    }
    return Ok(());
}

async fn get_path(store: &mut Store, path_name: &str) -> Res<()> {
    match store.get(path_name).await {
        Ok(path) => {
            println!("{}", path);
        }
        Err(err) => match err.downcast_ref::<sqlx::error::Error>() {
            Some(sqlx::error::Error::RowNotFound) => {
                return Err(anyhow!("Path '{}' not found.", path_name));
            }
            Some(_) | None => {
                return Err(err);
            }
        },
    };

    return Ok(());
}

async fn list_paths(store: &mut Store) -> Res<()> {
    let paths = store.list().await?;

    let max_name_len = paths
        .iter()
        .max_by_key(|it| it.name.len())
        .map(|it| it.name.len())
        .unwrap_or(0);

    for path in paths {
        println!("{:<len$} | {}", path.name, path.path, len = max_name_len);
    }

    Ok(())
}

async fn remove_path(store: &mut Store, name: &str) -> Res<()> {
    store.remove(name).await?;
    println!("Entry removed.");
    Ok(())
}
