mod store;
use clap::{ Parser, Subcommand };
use std::env;
use anyhow::anyhow;
use crate::store::Store;
use cli_table::{Cell, Table, format::{Border, Separator, Padding, Justify}, print_stdout};

type Res<T> = anyhow::Result<T>;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

}

#[derive(Subcommand, Debug)]
enum Commands {
    Save { name: Option<String>, },
    Get { name: String, },
    List,
    Remove { name: String, },
    Test,
}


#[tokio::main]
async fn main() -> Res<()> {
    let args = Args::parse();

    let mut store = Store::create("sqlite://test.db").await?;

    match args.command {
        Commands::Test => {

        }
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


async fn save_current_path(store: &mut Store, name: Option<&str>) -> Res<()> {
    let current_dir = env::current_dir()?;
    let dir_name = current_dir.file_name().unwrap().to_str().unwrap();
    let save_name = name.unwrap_or(dir_name);
    let save_path = current_dir.to_str().unwrap();
    match store.save(save_name, save_path).await {
        Ok(_) => {
        },
        Err(err) => {
            match err.downcast_ref::<sqlx::error::Error>() {
                Some(sqlx::error::Error::Database(db_err)) if db_err.is_unique_violation() => {
                    println!("Name '{}' is already used for another path.", save_name);
                },
                Some(_) | None => {
                    return Err(err);
                }
            }
        }
    }
    return Ok(())
}

async fn get_path(store: &mut Store, path_name: &str) -> Res<()> {
    match store.get(path_name).await {
        Ok(path) => {
            println!("{}", path);
        },
        Err(err) => {
            match err.downcast_ref::<sqlx::error::Error>() {
                Some(sqlx::error::Error::RowNotFound) => {
                    return Err(anyhow!("Path '{}' not found.", path_name));
                }
                Some(_) | None => {
                    return Err(err);
                }
            }
        }
    };

    return Ok(())
}

async fn list_paths(store: &mut Store) -> Res<()> {
    let paths = store.list().await?;

    let data: Vec<_> = paths.iter().map(|it| {
        vec![
            it.name.clone().cell()
                .justify(Justify::Left)
                .padding(Padding::builder().build()),
            it.path.clone().cell()]
    }).collect();

    let table = data.table()
        .border(Border::builder().build())
        .separator(Separator::builder().build());
    let _ = print_stdout(table);

    Ok(())
}

async fn remove_path(store: &mut Store, name: &str) -> Res<()> {
    store.remove(name).await?;
    println!("Entry removed.");
    Ok(())
}

