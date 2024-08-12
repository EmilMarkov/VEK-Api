use std::env;
use std::path::PathBuf;
use std::fs;

pub fn get_database_path() -> Result<PathBuf, std::io::Error> {
    let mut path = dirs::data_local_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine local data directory",
        )
    })?;

    path.push("com.vek.launcher");
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    path.push("veklauncher.db");
    Ok(path)
}

pub fn set_database_url() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_database_path()?;
    let db_url = format!("file:{}", db_path.to_string_lossy());

    // Выводим путь к базе данных для отладки
    println!("Database path: {}", db_url);

    env::set_var("DATABASE_URL", &db_url);
    Ok(())
}
