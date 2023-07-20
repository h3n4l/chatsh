use reqwest::blocking;
use serde::{Deserialize, Serialize};

use crate::converter::{Converter, Detail};

const PROMPT: &str =
        "Now, you are an assistant to help users to convert their text to shell commands\\
    (NOTE: The command may consist of multiple commands.).\\
    Your answer MUST be a json string (including other fields are DISALLOWED) and MUST ONLY contain the following two fields:
    1. descriptions: The array of string, each element interpreting a part of the command you generated. \\
    (For example, if you give the command [\"cd a\", \"ls -lh\"], the descriptions could be [\"`cd a`: navigate to a directory \", \"`ls -lh`: display the information about each item and its size\"]. \\
    NOTE: The description ALWAYS starts with the command you generated, and the command is wrapped by backticks.)
    2. commands: The array of command(s) meet the requirements.";

#[derive(Clone, Debug, Deserialize)]
struct PromptResponse {
    descriptions: Vec<String>,
    commands: Vec<String>,
}

pub struct GPT4_32K {
    client: blocking::Client,
    key: String,
}

impl GPT4_32K {
    pub fn new(key: &str) -> GPT4_32K {
        GPT4_32K {
            client: blocking::Client::new(),
            key: String::from(key),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct Request {
    model: String,
    messages: Vec<Message>,
    n: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Clone, Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

impl Converter for GPT4_32K {
    fn convert(&self, question: &str) -> anyhow::Result<Detail> {
        let request_body = serde_json::to_string(&Request {
            model: String::from("gpt-4-32k"),
            messages: vec![
                Message {
                    role: String::from("system"),
                    content: String::from(PROMPT),
                },
                Message {
                    role: String::from("user"),
                    content: String::from(question),
                },
            ],
            n: 1,
        })
        .expect("Failed to serialize request body.");

        let resp: Response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .bearer_auth(self.key.as_str())
            .body(request_body)
            .send()
            .expect("Failed to send request.")
            .json()
            .expect("Failed to deserialize response from gpt-4-32k.");

        if resp.choices.len() != 1 {
            return Err(anyhow::anyhow!(
                "Expect get one choice from gpt-4-32k response, but get {} choices.",
                resp.choices.len()
            ));
        }

        let choice = resp.choices[0].clone();

        let prompt_response: PromptResponse = serde_json::from_str(&choice.message.content)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to deserialize prompt response from gpt-4-32k, response: {}, err: {}",
                    choice.message.content,
                    e,
                )
            })?;

        Ok(Detail {
            descriptions: prompt_response.descriptions,
            command: prompt_response.commands.join(" && "),
        })
    }
}
