use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    system: Option<String>,
    options: OllamaOptions,
}

#[derive(Deserialize)]
pub struct OllamaResponse {
    pub response: String,
}

#[derive(Deserialize)]
struct StreamChunk {
    response: String,
    #[serde(default)]
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

    pub async fn ask(&self, system: &str, prompt: &str) -> anyhow::Result<String> {
        let req = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            system: Some(system.to_string()),
            options: OllamaOptions { temperature: 0.3, num_predict: 1024 },
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

    pub async fn ask_streaming(&self, system: &str, prompt: &str) -> anyhow::Result<String> {
        let req = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: true,
            system: Some(system.to_string()),
            options: OllamaOptions { temperature: 0.3, num_predict: 1024 },
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await;

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
        let mut line_buf = String::new();
        let mut raw_buf = String::new();
        let mut full_response = String::new();

        // Simple think-tag stripping for CLI output
        let mut in_think = false;
        let mut think_shown = false;

        while let Some(chunk) = stream.next().await {
            let bytes = match chunk { Ok(b) => b, Err(_) => break };
            let text = match std::str::from_utf8(&bytes) { Ok(s) => s, Err(_) => continue };
            line_buf.push_str(text);

            while let Some(pos) = line_buf.find('\n') {
                let line = line_buf[..pos].trim().to_string();
                line_buf.drain(..=pos);
                if line.is_empty() { continue; }

                if let Ok(chunk) = serde_json::from_str::<StreamChunk>(&line) {
                    if !chunk.response.is_empty() {
                        raw_buf.push_str(&chunk.response);
                    }

                    // Process raw_buf: handle <think> tags for terminal display
                    'process: loop {
                        if in_think {
                            if let Some(idx) = raw_buf.find("</think>") {
                                raw_buf.drain(..idx + 8);
                                in_think = false;
                                print!("\r                              \r");
                                io::stdout().flush().ok();
                            } else {
                                // Still inside think block — show indicator once
                                if !think_shown {
                                    print!("Thinking...");
                                    io::stdout().flush().ok();
                                    think_shown = true;
                                }
                                raw_buf.clear();
                                break 'process;
                            }
                        } else {
                            if let Some(idx) = raw_buf.find("<think>") {
                                // Flush anything before <think> as real output
                                let before = raw_buf[..idx].to_string();
                                if !before.is_empty() {
                                    print!("{}", before);
                                    io::stdout().flush().ok();
                                    full_response.push_str(&before);
                                }
                                raw_buf.drain(..idx + 7);
                                in_think = true;
                            } else {
                                // No tags — flush safe prefix
                                let safe = safe_prefix_len(&raw_buf, "<think>");
                                let to_print = &raw_buf[..safe];
                                if !to_print.is_empty() {
                                    print!("{}", to_print);
                                    io::stdout().flush().ok();
                                    full_response.push_str(to_print);
                                }
                                raw_buf.drain(..safe);
                                break 'process;
                            }
                        }
                    }

                    if chunk.done {
                        // Flush remainder
                        if !raw_buf.is_empty() && !in_think {
                            print!("{}", raw_buf);
                            io::stdout().flush().ok();
                            full_response.push_str(&raw_buf);
                            raw_buf.clear();
                        }
                        return Ok(full_response);
                    }
                }
            }
        }

        Ok(full_response)
    }
}

fn safe_prefix_len(s: &str, tag: &str) -> usize {
    if let Some(idx) = s.find(tag) { return idx; }
    let n = s.len();
    for i in (1..tag.len().min(n + 1)).rev() {
        if s.ends_with(&tag[..i]) { return n - i; }
    }
    n
}
