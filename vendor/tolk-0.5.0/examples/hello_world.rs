use std::io;

use tolk::Tolk;

fn main() -> io::Result<()> {
    let tolk = Tolk::new();
    tolk.try_sapi(true);
    println!(
        "Has speech: {}, has Braille: {}",
        tolk.has_speech(),
        tolk.has_braille()
    );
    tolk.output("Hello, world.", true);
    tolk.speak("This wil only speak.", false);
    tolk.braille("This will only be brailled.");
    if let Some(screen_reader) = tolk.detect_screen_reader() {
        println!("Detected screen reader: {}", screen_reader);
    } else {
        println!("No screen reader detected");
    }
    let mut _input = String::new();
    io::stdin().read_line(&mut _input)?;
    Ok(())
}
