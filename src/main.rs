use chatsh::app;
use chrono::Local;
use std::env;
use std::io::Write;

fn main() {
    init_logger();
    let openai_key = env::var("OPENAI_KEY");
    if openai_key.is_err() {
        log::error!(
            "Cannot get OPENAI_KEY environment variable, error: {}",
            openai_key.err().unwrap()
        );
        std::process::exit(1);
    }

    let openai_key = openai_key.unwrap();
    let mut app = app::App::new(openai_key.as_str());
    app.run();
}

fn init_logger() {
    env_logger::Builder::new()
        .format(|buf, record| {
            let mut level_style = buf.style();
            match record.level() {
                log::Level::Error => level_style
                    .set_color(env_logger::fmt::Color::Red)
                    .set_bold(true),
                log::Level::Warn => level_style
                    .set_color(env_logger::fmt::Color::Yellow)
                    .set_bold(true),
                log::Level::Info => level_style
                    .set_color(env_logger::fmt::Color::Green)
                    .set_bold(true),
                log::Level::Debug => level_style
                    .set_color(env_logger::fmt::Color::Blue)
                    .set_bold(true),
                log::Level::Trace => level_style
                    .set_color(env_logger::fmt::Color::Magenta)
                    .set_bold(true),
            };
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                level_style.value(record.level()),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Info)
        .init();
}
