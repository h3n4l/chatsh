use std::fmt;

use crate::converter::{openai, Converter, Detail};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};

enum Status {
    Begin(String),
    WaitingText,
    WaitingUserChoice,
    End,
}

enum UserChoice {
    ExecuteCommand,
    EditAndRunCommand,
    AskAnotherQuestion,
    Cancel,
}

impl fmt::Display for UserChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            UserChoice::ExecuteCommand => write!(f, "Execute the command directly."),
            UserChoice::EditAndRunCommand => write!(f, "Edit and run the command."),
            UserChoice::AskAnotherQuestion => write!(f, "Ask another question."),
            UserChoice::Cancel => write!(f, "Cancel."),
        }
    }
}

pub struct App {
    converter: Box<dyn Converter>,
    status: Status,
    last_detail: Option<Detail>,
}

impl App {
    pub fn new(key: &str) -> App {
        App {
            converter: Box::new(openai::gpt35_turbo::GPT35Turbo::new(key)),
            status: Status::Begin("Convert your text to shell commands".bold().to_string()),
            last_detail: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            match &self.status {
                Status::Begin(hello) => {
                    println!("{}", hello);
                    self.status = Status::WaitingText;
                }
                Status::WaitingText => {
                    let question = Text::new("Text: ")
                        .with_help_message("Input 'exit' or 'quit' to end the program.")
                        .prompt()
                        .unwrap();

                    if question == "exit" || question == "quit" {
                        self.status = Status::End;
                        continue;
                    }

                    let spinner_style =
                        ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
                            .unwrap()
                            .tick_chars("⠁⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉⠈⠈✔");
                    let progress_bar = ProgressBar::new_spinner();
                    progress_bar.set_style(spinner_style);
                    progress_bar.enable_steady_tick(std::time::Duration::from_millis(25));
                    progress_bar.set_prefix("Converting");

                    let detail = self.converter.convert(&question);
                    progress_bar.finish_and_clear();
                    if let Err(e) = detail {
                        println!("Error: {}", e.to_string().red());
                        continue;
                    }
                    let detail = detail.unwrap();
                    println!("{}:", "Description".bold());
                    for (index, desc) in detail.descriptions.iter().enumerate() {
                        println!("{}. {}", (index + 1).to_string().bold(), desc);
                    }
                    println!("{}: {}", "Command".bold(), detail.command.bold());
                    self.last_detail = Some(detail);
                    self.status = Status::WaitingUserChoice;
                }
                Status::WaitingUserChoice => {
                    let last_detail = self.last_detail.as_ref().unwrap();

                    let options = vec![
                        UserChoice::ExecuteCommand,
                        UserChoice::EditAndRunCommand,
                        UserChoice::AskAnotherQuestion,
                        UserChoice::Cancel,
                    ];
                    let ans = Select::new("What do you want to do next?", options)
                        .prompt()
                        .unwrap();
                    match ans {
                        UserChoice::ExecuteCommand => {
                            let result = execute_command(last_detail.command.as_str());
                            match result {
                                Ok(_) => {
                                    self.status = Status::End;
                                }
                                Err(e) => {
                                    println!("Error: {}", e.to_string().red());
                                    self.status = Status::WaitingText;
                                    self.last_detail = None;
                                    continue;
                                }
                            }
                        }
                        UserChoice::EditAndRunCommand => {
                            let new_command = Text::new("Command: ")
                                .with_initial_value(last_detail.command.as_str())
                                .prompt()
                                .unwrap();

                            let result = execute_command(new_command.as_str());
                            match result {
                                Ok(_) => {
                                    self.status = Status::End;
                                }
                                Err(e) => {
                                    println!("Error: {}", e.to_string().red());
                                    self.status = Status::WaitingText;
                                    self.last_detail = None;
                                    continue;
                                }
                            }
                        }
                        UserChoice::AskAnotherQuestion => {
                            self.status = Status::WaitingText;
                            self.last_detail = None;
                        }
                        UserChoice::Cancel => {
                            self.status = Status::End;
                        }
                    }
                }
                Status::End => {
                    return;
                }
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn execute_command(command: &str) -> anyhow::Result<()> {
    let output = std::process::Command::new("sh")
        .args(["-c", command])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Command failed with exit code {}",
            output.status.code().unwrap()
        ))
    }
}
