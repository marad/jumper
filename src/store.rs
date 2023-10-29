use sqlx::{SqlitePool, Pool, Sqlite, FromRow, migrate::MigrateDatabase};
use crate::Res;

#[derive(Clone, FromRow, Debug)]
pub struct SavedPath {
    pub name: String,
    pub path: String,
}


#[derive(Debug)]
pub struct Store {
    pool: Pool<Sqlite>
}

impl Store {

    pub async fn create(url: &str) -> Res<Store> {
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

    pub async fn save(&mut self, name: &str, path: &str) -> Res<()> {
        sqlx::query("INSERT INTO paths (name, path) VALUES (?, ?)")
            .bind(name)
            .bind(path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get(&mut self, name: &str) -> Res<String> {
        let path = sqlx::query_as::<_, SavedPath>("SELECT name, path FROM paths WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await?;
        Ok(path.path)
    }


    pub async fn list(&mut self) -> Res<Vec<SavedPath>> {
        let data = sqlx::query_as::<_, SavedPath>("SELECT name, path FROM paths")
            .fetch_all(&self.pool)
            .await?;
        Ok(data)
    }

    pub async fn remove(&mut self, name: &str) -> Res<()> {
        sqlx::query("DELETE FROM paths where name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
