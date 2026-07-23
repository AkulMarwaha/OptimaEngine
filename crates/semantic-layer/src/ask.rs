use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::channel::mpsc;
use futures::SinkExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

use crate::AppState;

#[derive(Deserialize)]
pub struct AskRequest {
    pub question: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaChunk {
    response: String,
    #[serde(default)]
    done: bool,
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
                let _ = tx.send(Ok(Event::default().data(msg))).await;
                let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
                return;
            }
            Err(e) => {
                let msg = format!("Cannot reach Ollama — is it running? ({})", e);
                let _ = tx.send(Ok(Event::default().data(msg))).await;
                let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
                return;
            }
        };

        let mut stream = res.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let bytes = match chunk {
                Ok(b) => b,
                Err(_) => break,
            };
            let text = match std::str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => continue,
            };
            buffer.push_str(text);

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer.drain(..=pos);

                if line.is_empty() {
                    continue;
                }

                if let Ok(chunk) = serde_json::from_str::<OllamaChunk>(&line) {
                    if !chunk.response.is_empty() {
                        // JSON-encode each token so SSE doesn't strip leading spaces
                        let payload = serde_json::json!({"t": chunk.response}).to_string();
                        if tx.send(Ok(Event::default().data(payload))).await.is_err() {
                            return;
                        }
                    }
                    if chunk.done {
                        let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
                        return;
                    }
                }
            }
        }

        let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
    });

    Sse::new(rx).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
