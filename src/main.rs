use sqlx::{SqlitePool, Pool, Sqlite, FromRow, migrate::MigrateDatabase};
use clap::{ Parser, Subcommand };
use std::env;
use anyhow::anyhow;

type Res<T> = anyhow::Result<T>;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

}

#[derive(Subcommand, Debug)]
enum Commands {
    Save {
        name: Option<String>,
    },
    Get {
        name: String,
    },
    List,
    Remove {
        name: String,
    }
}


#[tokio::main]
async fn main() -> Res<()> {
    let args = Args::parse();

    let mut store = Store::create("sqlite://test.db").await?;

    match args.command {
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
    store.save(save_name, save_path).await?;
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
    // TODO: handle showing the table nicely with different name lengths
    let paths = store.list().await?;
    
    for path in paths {
        println!("{}\t{}", path.name, path.path);
    }

    Ok(())
}

async fn remove_path(store: &mut Store, name: &str) -> Res<()> {
    store.remove(name).await?;
    println!("Entry removed.");
    Ok(())
}

#[derive(Clone, FromRow, Debug)]
struct SavedPath {
    name: String,
    path: String,
}


#[derive(Debug)]
struct Store {
    pool: Pool<Sqlite>
}

impl Store {

    async fn create(url: &str) -> Res<Store> {
        if !Sqlite::database_exists(url).await.unwrap_or(false) {
            Sqlite::create_database(url).await?;
        }
        let pool = SqlitePool::connect(url).await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS paths (
                id INTEGER PRIMARY KEY NOT NULL,
                name VARCHAR(50) UNIQUE NOT NULL,
                path VARCHAR(500))")
            .execute(&pool).await?;
        return Ok(Store { pool });
    }

    async fn save(&mut self, name: &str, path: &str) -> Res<()> {
        // TODO: handle unique constraint error
        sqlx::query("INSERT INTO paths (name, path) VALUES (?, ?)")
            .bind(name)
            .bind(path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get(&mut self, name: &str) -> Res<String> {
        let path = sqlx::query_as::<_, SavedPath>("SELECT name, path FROM paths WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await?;
        Ok(path.path)
    }


    async fn list(&mut self) -> Res<Vec<SavedPath>> {
        let data = sqlx::query_as::<_, SavedPath>("SELECT name, path FROM paths")
            .fetch_all(&self.pool)
            .await?;
        Ok(data)
    }

    async fn remove(&mut self, name: &str) -> Res<()> {
        sqlx::query("DELETE FROM paths where name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
