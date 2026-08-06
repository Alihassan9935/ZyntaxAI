use std::time::Duration;

fn main() {
    let mut args = std::env::args().skip(1);

    let text = args.next().unwrap_or_else(|| {
        eprintln!("usage: seed_clipboard <text> [seconds-to-hold]");
        std::process::exit(2);
    });
    let hold: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(60);

    let mut clipboard = zyntax_platform::Clipboard::new().expect("clipboard unavailable");
    clipboard.set_text(&text).expect("could not set clipboard");

    println!("clipboard seeded; holding for {hold}s");
    std::thread::sleep(Duration::from_secs(hold));
}
