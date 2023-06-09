use colored::*;
use std::env;

use chatsh::app;

fn main() {
    let openai_key = env::var("OPENAI_KEY");
    if openai_key.is_err() {
        println!(
            "{}",
            format!(
                "Cannot get OPENAI_KEY environment variable, error: {}",
                openai_key.err().unwrap()
            )
            .red()
        );
        std::process::exit(1);
    }

    let openai_key = openai_key.unwrap();
    let mut app = app::App::new(openai_key.as_str());
    app.run();
}
