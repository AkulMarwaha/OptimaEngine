use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::channel::mpsc;
use futures::SinkExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::{Duration, Instant}};

use crate::AppState;

#[derive(Deserialize)]
pub struct AskRequest {
    pub question: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Deserialize)]
struct OllamaChunk {
    response: String,
    #[serde(default)]
    done: bool,
}

// SSE events are JSON objects: {"type": "...", ...}
// Types: thinking_start | thinking | thinking_done | answer | direct | done

/// Returns the number of bytes from the start of `s` that are safe to emit —
/// i.e. cannot be the beginning of `tag` at the tail of `s`.
fn safe_prefix_len(s: &str, tag: &str) -> usize {
    if let Some(idx) = s.find(tag) { return idx; }
    let n = s.len();
    for i in (1..tag.len().min(n + 1)).rev() {
        if s.ends_with(&tag[..i]) { return n - i; }
    }
    n
}

enum Phase {
    /// Watching for <think>. Accumulates all tokens until we see the tag or give up.
    Scanning { buf: String, token_count: usize },
    /// Inside <think>...</think>.
    Thinking { buf: String, start: Instant },
    /// After </think>, streaming the final answer.
    Answer,
    /// No <think> appeared in the first 50 tokens — direct answer mode.
    Direct,
}

pub async fn ask(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AskRequest>,
) -> Sse<impl futures::Stream<Item = Result<Event, anyhow::Error>>> {
    let (mut tx, rx) = mpsc::channel::<Result<Event, anyhow::Error>>(64);

    tokio::spawn(async move {
        let system = state.system_prompt.clone();
        let prompt = format!("Question: {}", body.question);

        let req_body = OllamaRequest {
            model: state.ollama_model.clone(),
            prompt,
            system,
            stream: true,
            options: OllamaOptions {
                temperature: 0.3,
                num_predict: 1024,
            },
        };

        let client = reqwest::Client::new();
        let res = match client
            .post(format!("{}/api/generate", state.ollama_url))
            .json(&req_body)
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                let msg = format!("Ollama returned HTTP {}", r.status());
                let _ = tx.send(Ok(Event::default().data(
                    serde_json::json!({"type":"direct","t":msg}).to_string()
                ))).await;
                let _ = tx.send(Ok(Event::default().data(
                    serde_json::json!({"type":"done"}).to_string()
                ))).await;
                return;
            }
            Err(e) => {
                let msg = format!("Cannot reach Ollama — is it running? ({})", e);
                let _ = tx.send(Ok(Event::default().data(
                    serde_json::json!({"type":"direct","t":msg}).to_string()
                ))).await;
                let _ = tx.send(Ok(Event::default().data(
                    serde_json::json!({"type":"done"}).to_string()
                ))).await;
                return;
            }
        };

        let mut stream = res.bytes_stream();
        let mut line_buf = String::new();
        let mut phase = Phase::Scanning { buf: String::new(), token_count: 0 };

        macro_rules! send {
            ($payload:expr) => {
                if tx.send(Ok(Event::default().data($payload.to_string()))).await.is_err() {
                    return;
                }
            };
        }

        while let Some(chunk) = stream.next().await {
            let bytes = match chunk { Ok(b) => b, Err(_) => break };
            let text = match std::str::from_utf8(&bytes) { Ok(s) => s, Err(_) => continue };
            line_buf.push_str(text);

            while let Some(pos) = line_buf.find('\n') {
                let line = line_buf[..pos].trim().to_string();
                line_buf.drain(..=pos);
                if line.is_empty() { continue; }

                let ollama_chunk = match serde_json::from_str::<OllamaChunk>(&line) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let token = ollama_chunk.response;
                let is_done = ollama_chunk.done;

                // Run the state machine for this token
                // We use a loop so a state transition can immediately re-process leftovers.
                let mut leftovers: Option<String> = if token.is_empty() { None } else { Some(token) };
                loop {
                    let token_str = leftovers.take().unwrap_or_default();

                    let new_phase = match &mut phase {
                        Phase::Scanning { buf, token_count } => {
                            buf.push_str(&token_str);
                            *token_count += 1;
                            let cnt = *token_count;

                            if let Some(idx) = buf.find("<think>") {
                                // Discard preamble, switch to Thinking
                                let thinking_buf = buf[idx + 7..].to_string();
                                send!(serde_json::json!({"type":"thinking_start"}));
                                Some(Phase::Thinking { buf: thinking_buf, start: Instant::now() })
                            } else if cnt >= 50 {
                                // Give up waiting — flush buffer as direct
                                let content = std::mem::take(buf);
                                if !content.is_empty() {
                                    send!(serde_json::json!({"type":"direct","t":content}));
                                }
                                Some(Phase::Direct)
                            } else {
                                None // keep scanning
                            }
                        }

                        Phase::Thinking { buf, start } => {
                            buf.push_str(&token_str);

                            if let Some(idx) = buf.find("</think>") {
                                // Found closing tag
                                let content = buf[..idx].to_string();
                                let after = buf[idx + 8..].to_string();
                                if !content.is_empty() {
                                    send!(serde_json::json!({"type":"thinking","t":content}));
                                }
                                let elapsed = start.elapsed().as_secs();
                                send!(serde_json::json!({"type":"thinking_done","elapsed":elapsed}));
                                // Any text after </think> becomes the first answer chunk
                                if !after.is_empty() {
                                    leftovers = Some(after);
                                }
                                Some(Phase::Answer)
                            } else {
                                // Stream safe thinking content
                                let safe = safe_prefix_len(buf, "</think>");
                                if safe > 0 {
                                    let to_send = buf[..safe].to_string();
                                    buf.drain(..safe);
                                    send!(serde_json::json!({"type":"thinking","t":to_send}));
                                }
                                None
                            }
                        }

                        Phase::Answer => {
                            if !token_str.is_empty() {
                                send!(serde_json::json!({"type":"answer","t":token_str}));
                            }
                            None
                        }

                        Phase::Direct => {
                            if !token_str.is_empty() {
                                send!(serde_json::json!({"type":"direct","t":token_str}));
                            }
                            None
                        }
                    };

                    if let Some(p) = new_phase {
                        phase = p;
                        if leftovers.is_some() { continue; } // re-process leftovers in new state
                    }
                    break;
                }

                if is_done {
                    match &mut phase {
                        Phase::Scanning { buf, .. } => {
                            let content = std::mem::take(buf);
                            if !content.is_empty() {
                                send!(serde_json::json!({"type":"direct","t":content}));
                            }
                        }
                        Phase::Thinking { buf, start } => {
                            if !buf.is_empty() {
                                let content = std::mem::take(buf);
                                let elapsed = start.elapsed().as_secs();
                                send!(serde_json::json!({"type":"thinking","t":content}));
                                send!(serde_json::json!({"type":"thinking_done","elapsed":elapsed}));
                            }
                        }
                        _ => {}
                    }
                    send!(serde_json::json!({"type":"done"}));
                    return;
                }
            }
        }

        send!(serde_json::json!({"type":"done"}));
    });

    Sse::new(rx).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
