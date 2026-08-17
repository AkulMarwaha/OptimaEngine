# Optima Engine — Claude Code Context

## What this is
Optima Engine is an on-premises AI intelligence layer for mid-market 
automotive suppliers running any ERP system. It ingests ERP data, runs a 
Bronze→Silver→Gold medallion pipeline, and answers natural language 
questions about margin, delivery, and budget performance. Everything 
runs locally — no data leaves the building. Built in Rust.

## Repo
- GitHub: https://github.com/AkulMarwaha/OptimaEngine
- Active branch: develop
- Local path: /Users/akulmarwaha/Documents/OptimaEngine

## Workspace structure
- crates/core — shared models, config, error types
- crates/ingestion — Mockaroo connector → Bronze CSVs
- crates/transformation — Polars pipeline → Silver + Gold parquet
- crates/semantic-layer — Axum REST API serving Gold data (port 3000)
- crates/ai-agent — natural language interface via Ollama
- crates/vector-store — planned, not built yet
- data/bronze/ — raw CSVs
- data/silver/ — enriched parquet
- data/gold/ — aggregated analytics parquet

## Tech stack
- Rust workspace, edition 2024, resolver 2
- Polars 0.46 for data transformation
- Axum 0.7 for REST API
- Tokio 1.x async runtime
- Ollama local inference (model: qwen2.5:1.5b, port 11434)
- Mockaroo for synthetic SAP data

## Critical Rust rules — never break these
- Polars 0.46: join() requires 5 arguments, None as last arg
- Use lazy group_by().agg() for deduplication, never unique()
- optima-core alias required in every crate's Cargo.toml to avoid 
  conflict with Rust stdlib core crate
- Import as: use optima_core::...
- Never touch Bronze/Silver/Gold pipeline code unless explicitly asked

## Pipeline run commands
RUST_LOG=info cargo run -p ingestion
RUST_LOG=info cargo run -p transformation
RUST_LOG=info cargo run -p semantic-layer
RUST_LOG=info cargo run -p ai-agent

## Check before committing
cargo check -p core && cargo check -p ingestion && cargo check -p transformation && cargo check -p semantic-layer && cargo check -p ai-agent

## Git workflow
- Always work on develop branch
- Commit format: feat(crate-name): description
- Always push to origin develop after committing

## Ollama
- Running locally at http://localhost:11434
- Model: qwen2.5:1.5b (already pulled)
- Never use any cloud APIs — everything must be local

## Current state
- Bronze: complete and working
- Silver: complete and working
- Gold: complete and working
- Semantic Layer API: built, needs verification
- AI Agent: built, needs verification
- Vector store: not started

## Configurable Pipeline Plan

### Goal
Self-serve onboarding: a company connects their own ERP export without
ever sending data or schema information externally. Everything stays
on-premises.

### Onboarding flow
1. **Upload** — company uploads a CSV or Excel file through the web UI
2. **Propose mapping** — system proposes a column mapping via a two-tier
   matcher:
   - Tier 1: known ERP field codes (e.g. SAP: VBELN, KUNNR, MATNR, ...)
     matched by exact code lookup
   - Tier 2: heuristic matching on header wording, used as fallback for
     any ERP not in the known-codes list
3. **Confirm/edit** — proposed mapping is shown on a review screen;
   a human approves or corrects it before anything runs
4. **Save & run** — confirmed mapping is saved as a config file; every
   future export from that company runs against the saved config
   automatically — no re-mapping needed

### Design constraints
- Must work with any ERP, not just SAP
- SAP is the first supported pattern set, not a ceiling
- The canonical generic field names used internally are already in place
  (see transformation crate — order_id, material_id, department, etc.)

### Implementation status
| Component | Status |
|-----------|--------|
| Generic internal field names | Done (transformation crate) |
| Field matching logic | Not built |
| Upload endpoint | Not built |
| Confirm/edit screen | Not built |
| Pipeline wiring to saved config | Not built |
