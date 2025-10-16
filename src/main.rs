use windows::Globalization::ApplicationLanguages;

fn main() -> windows::core::Result<()> {
    let languages = ApplicationLanguages::Languages()?;
    
    for lang in languages {
        println!("- {}", lang);
    }

    Ok(())
}
