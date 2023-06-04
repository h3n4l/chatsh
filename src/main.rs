use std::env;

use chatsh::app;

fn main() {
    let openai_key = env::var("OPENAI_KEY").expect("OPENAI_KEY is not set.");
    let mut app = app::App::new(openai_key.as_str());
    app.run();
}
