use windows::{Storage::StorageFile, core::HSTRING};

fn main() -> windows::core::Result<()> {
    let startup_path = std::env::current_exe()?;

    let path_hstring = HSTRING::from(startup_path.to_string_lossy().to_string());
    println!("Startup Path: {}", path_hstring);

    let file_async = StorageFile::GetFileFromPathAsync(&path_hstring)?;
    match file_async.join() {
        Ok(file) => {
            let file_path = file.Path()?;
            println!("StorageFile Path: {}", file_path);
        }
        Err(e) => {
            println!("Error retrieving StorageFile: {}", e);
        }
    }

    Ok(())
}
