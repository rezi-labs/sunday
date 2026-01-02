use tokio_postgres::Client;

pub async fn run_migrations(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    // Create migrations table to track which migrations have been run
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS migrations (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL UNIQUE,
            executed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
            &[],
        )
        .await?;

    // Get list of already executed migrations
    let rows = client
        .query("SELECT name FROM migrations ORDER BY name", &[])
        .await?;
    let executed_migrations: Vec<String> = rows.iter().map(|row| row.get("name")).collect();

    // Migration files in order
    let migrations = vec![
        (
            "001_initial_schema.sql",
            include_str!("001_initial_schema.sql"),
        ),
        (
            "003_make_year_required.sql",
            include_str!("003_make_year_required.sql"),
        ),
    ];

    for (name, sql) in migrations {
        if !executed_migrations.contains(&name.to_string()) {
            log::info!("Running migration: {name}");

            // Execute the migration
            client.batch_execute(sql).await?;

            // Record that this migration was executed
            client
                .execute("INSERT INTO migrations (name) VALUES ($1)", &[&name])
                .await?;

            log::info!("Migration {name} completed successfully");
        } else {
            log::debug!("Migration {name} already executed, skipping");
        }
    }

    log::info!("All migrations completed successfully");
    Ok(())
}
