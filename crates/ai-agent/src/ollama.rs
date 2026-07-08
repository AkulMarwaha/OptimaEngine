use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub system: Option<String>,
}

#[derive(Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

#[derive(Deserialize)]
struct StreamChunk {
    response: String,
    #[serde(default)]
    thinking: Option<String>,
    done: bool,
}

pub struct OllamaClient {
    pub base_url: String,
    pub model: String,
    pub client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Non-streaming call — used for routing (classification) only.
    pub async fn ask(&self, system: &str, prompt: &str) -> anyhow::Result<String> {
        let req = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            system: Some(system.to_string()),
        };

        let res = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(res.response.trim().to_string())
    }

    /// Streaming call — prints tokens to stdout as they arrive.
    /// Shows a thinking indicator while Qwen3 reasons internally,
    /// then clears it and streams the actual response word by word.
    /// Falls back to ask() if the streaming request fails.
    pub async fn ask_streaming(&self, system: &str, prompt: &str) -> anyhow::Result<String> {
        // Append /no_think to system prompt to force Qwen3 to skip
        // its internal reasoning phase and stream the answer directly.
        let system_with_no_think = format!("{}\n/no_think", system);

        let req = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: true,
            system: Some(system_with_no_think),
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await;

        // If request failed or returned bad status, fall back to non-streaming
        let response = match response {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                tracing::warn!("Streaming got HTTP {}, falling back to ask()", r.status());
                let answer = self.ask(system, prompt).await?;
                print!("{}", answer);
                io::stdout().flush().ok();
                return Ok(answer);
            }
            Err(e) => {
                tracing::warn!("Streaming request failed ({}), falling back to ask()", e);
                let answer = self.ask(system, prompt).await?;
                print!("{}", answer);
                io::stdout().flush().ok();
                return Ok(answer);
            }
        };

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut full_response = String::new();
        let mut thinking_shown = false;
        let mut response_started = false;

        while let Some(chunk) = stream.next().await {
            let bytes = match chunk {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("Streaming chunk error: {}", e);
                    break;
                }
            };

            let text = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => continue,
            };
            buffer.push_str(text);

            // Process every complete newline-delimited JSON object in the buffer
            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer.drain(..=pos);

                if line.is_empty() {
                    continue;
                }

                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(&line) {

                    // Qwen3 is still thinking — response field is empty
                    if chunk.response.is_empty() && !chunk.done {
                        if let Some(ref t) = chunk.thinking {
                            if !t.is_empty() && !thinking_shown {
                                print!("analyzing...");
                                io::stdout().flush().ok();
                                thinking_shown = true;
                            }
                        }
                        continue;
                    }

                    // First real response token — clear the thinking indicator
                    if !chunk.response.is_empty() && !response_started {
                        if thinking_shown {
                            print!("\r                              \r");
                            io::stdout().flush().ok();
                        }
                        response_started = true;
                    }

                    // Stream the actual response token
                    if !chunk.response.is_empty() {
                        print!("{}", chunk.response);
                        io::stdout().flush().ok();
                        full_response.push_str(&chunk.response);
                    }

                    if chunk.done {
                        return Ok(full_response);
                    }
                }
            }
        }

        Ok(full_response)
    }
}