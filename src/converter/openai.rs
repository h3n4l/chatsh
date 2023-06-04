pub mod gpt35_turbo {
    use reqwest::blocking;
    use serde::{Deserialize, Serialize};

    use crate::converter::{Converter, Detail};

    const PROMPT: &str =
        "Now, you are an assistant to help users to convert their text to shell commands\\
    (NOTE: The command may consist of multiple commands.).\\
    Your answer MUST be a json string (including other descriptions is DISALLOWED) and MUST contain the following fields:
    1. description: What is each part of this command doing? It should be as short as possible.
    2. command: The command(s) meet the requirements.";

    #[derive(Clone, Debug, Deserialize)]
    struct PromptResponse {
        description: String,
        command: String,
    }

    pub struct GPT35Turbo {
        client: blocking::Client,
        key: String,
    }

    impl GPT35Turbo {
        pub fn new(key: &str) -> GPT35Turbo {
            GPT35Turbo {
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

    impl Converter for GPT35Turbo {
        fn convert(&self, question: &str) -> anyhow::Result<Detail> {
            let request_body = serde_json::to_string(&Request {
                model: String::from("gpt-3.5-turbo"),
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
                .expect("Failed to deserialize response from gpt-3.5-turbo.");

            if resp.choices.len() != 1 {
                return Err(anyhow::anyhow!(
                    "Expect get one choice from gpt-3.5-turbo response, but get {} choices.",
                    resp.choices.len()
                ));
            }

            let choice = resp.choices[0].clone();

            let prompt_response: PromptResponse = serde_json::from_str(&choice.message.content)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to deserialize prompt response from gpt-3.5-turbo response: {}, err: {}",
                        choice.message.content,
                        e,
                    )
                })?;

            Ok(Detail {
                description: prompt_response.description,
                command: prompt_response.command,
            })
        }
    }
}
