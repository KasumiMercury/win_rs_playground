use windows::{core::HSTRING, Globalization::ApplicationLanguages, Storage::StorageFile};

fn main() -> windows::core::Result<()> {
    let startup_path = std::env::current_exe()?;

    let path_hstring = HSTRING::from(startup_path.to_string_lossy().to_string());
    println!("Startup Path: {}", path_hstring);

    let _file = StorageFile::GetFileFromPathAsync(&path_hstring)?;

    let languages = ApplicationLanguages::Languages()?;
    
    for lang in languages {
        println!("- {}", lang);
    }

    Ok(())
}
