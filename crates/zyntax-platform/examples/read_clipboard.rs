fn main() {
    let mut clipboard = zyntax_platform::Clipboard::new().expect("clipboard unavailable");
    match clipboard.text() {
        Ok(text) => println!("{text}"),
        Err(err) => {
            eprintln!("clipboard: {err}");
            std::process::exit(1);
        }
    }
}
