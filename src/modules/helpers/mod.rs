use std::fs::{self};
use std::path::PathBuf;
use std::env;

use crate::modules::formatters::{
    remove_duplicate_spaces,
    remove_release_year_from_name,
    remove_special_edition_from_name,
    remove_symbols_from_name,
    remove_trash,
};

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

    println!("Database path: {}", db_url);

    env::set_var("DATABASE_URL", &db_url);
    Ok(())
}

pub fn pipe<T, F>(input: T, functions: Vec<F>) -> T
where
    F: Fn(T) -> T,
{
    functions.into_iter().fold(input, |acc, func| func(acc))
}

pub fn format_name(name: String) -> String {
    let functions: Vec<fn(String) -> String> = vec![
        remove_trash,
        remove_release_year_from_name,
        remove_symbols_from_name,
        remove_special_edition_from_name,
        remove_duplicate_spaces,
    ];
    pipe(name, functions).trim().to_string()
}
