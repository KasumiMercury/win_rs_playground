use windows::{
    core::HSTRING,
    UI::Popups::{MessageDialog, UICommand},
};

fn main() -> windows::core::Result<()> {
    let content = HSTRING::from("test");
    let title = HSTRING::from("test");
    let dialog = MessageDialog::CreateWithTitle(&content, &title)?;

    let yes_label = HSTRING::from("はい");
    let yes_command = UICommand::Create(&yes_label)?;
    dialog.Commands()?.Append(&yes_command)?;

    let no_label = HSTRING::from("いいえ");
    let no_command = UICommand::Create(&no_label)?;
    dialog.Commands()?.Append(&no_command)?;

    dialog.SetDefaultCommandIndex(0)?;
    dialog.SetCancelCommandIndex(1)?;

    let result = dialog.ShowAsync()?.join()?;

    println!("{}", result.Label()?);

    Ok(())
}
