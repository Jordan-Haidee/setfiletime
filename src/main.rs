use chrono::{NaiveDateTime, TimeZone};
use clap::{Arg, Command};
use filetime::{FileTime, set_file_times};
use std::path::Path;
use walkdir::WalkDir;

fn parse_args() -> (String, String) {
    let matches = Command::new("A simple utility to update file timestamps")
        .arg(
            Arg::new("path")
                .help("Path to file or directory")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("time")
                .help("Timestamp to set in format \"YYYY-MM-DD HH:MM:SS\"")
                .required(true)
                .index(2),
        )
        .get_matches();

    let path_str: &String = matches.get_one("path").expect("Path is required");
    let time_str: &String = matches.get_one("time").expect("Time is required");
    return (path_str.clone(), time_str.clone());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let (path_str, time_str) = parse_args();

    // Parse datetime string
    let datetime = NaiveDateTime::parse_from_str(time_str.as_str(), "%Y-%m-%d %H:%M:%S")?;
    let local_datetime = chrono::Local
        .from_local_datetime(&datetime)
        .single()
        .ok_or("Invalid or ambiguous datetime")?;

    // Convert to filetime type
    let system_time = std::time::SystemTime::from(local_datetime);
    let file_time = FileTime::from_system_time(system_time);

    let path = Path::new(path_str.as_str());

    // Check if path exists
    if !path.exists() {
        return Err(format!("Path '{}' does not exist", path.display()).into());
    }

    // Process files/directories
    if path.is_file() {
        set_file_times(path, file_time, file_time)?;
        println!("Updated timestamp for: {}", path.display());
    } else if path.is_dir() {
        // Walk through directory recursively
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if entry.file_type().is_symlink() {
                continue;
            }
            match set_file_times(entry_path, file_time, file_time) {
                Ok(_) => println!("Updated timestamp for: {}", entry_path.display()),
                Err(e) => eprintln!("Failed to update {}: {}", entry_path.display(), e),
            }
        }
    } else {
        return Err("Path is neither a file nor a directory".into());
    }

    Ok(())
}
