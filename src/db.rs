use sqlx::{migrate::MigrateDatabase, FromRow};

const DB_URL: &str = "sqlite://questrade_asset_tracker.db";

#[derive(Clone, FromRow, Debug)]
pub struct RefreshToken {
    id: i64,
    pub refresh_token: String,
}

pub struct DatabaseAPI {
    pool: sqlx::sqlite::SqlitePool,
}

impl DatabaseAPI {
    pub async fn new() -> Result<Self, sqlx::Error> {
        if !sqlx::sqlite::Sqlite::database_exists(DB_URL)
            .await
            .unwrap_or(false)
        {
            sqlx::sqlite::Sqlite::create_database(DB_URL).await?;
            println!("Created a new database");
        }

        let pool = sqlx::sqlite::SqlitePool::connect(DB_URL).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS refresh_token (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            refresh_token VARCHAR(64) NOT NULL);",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn get_refresh_token(&self) -> Result<RefreshToken, sqlx::Error> {
        let token = sqlx::query_as::<_, RefreshToken>("SELECT * FROM refresh_token")
            .fetch_one(&self.pool)
            .await?;

        Ok(token)
    }

    pub async fn insert_refresh_token(&self, refresh_token: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO refresh_token (refresh_token) VALUES (?)")
            .bind(refresh_token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_refresh_token(
        &self,
        refresh_token: &RefreshToken,
        new_value: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE refresh_token SET refresh_token = ? WHERE id = ?")
            .bind(new_value)
            .bind(refresh_token.id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
