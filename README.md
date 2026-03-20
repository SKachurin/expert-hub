!# Expert Hub

## Current stage

Expert Hub is currently a Rust-based Telegram-first MVP shell for an expert marketplace.

At this stage, the project already has:

- Rust backend with `actix-web`
- PostgreSQL in Docker
- SeaORM added as the ORM / entity / migration direction
- Telegram web auth fallback for normal browser context
- Telegram Mini App context support
- TON wallet connection on the frontend
- marketplace main page
- expert onboarding page
- basic backend endpoints for Telegram auth verification and wallet linking

The current codebase is not yet a full booking platform. Right now it is mainly:

1. project skeleton
2. Docker runtime
3. frontend host for web and Telegram app context
4. Telegram identity handling
5. TON wallet connection handling
6. database structure draft for MVP entities

---

## Project structure

Main project files currently in use:

- `Cargo.toml`
- `Cargo.lock`
- `docker-compose.dev.yml`
- `docker-compose.prod.yml`
- `Dockerfile.dev`
- `Dockerfile.prod`
- `Makefile`
- `src/main.rs`
- `public/index.html`
- `public/expert-new.html`
- `scripts/init-dev.sql`
- `scripts/init-prod.sql`

Earlier project structure and Docker setup were documented in the exported project snapshot. :contentReference[oaicite:0]{index=0}

---

## Tech stack currently used

### Backend
- Rust
- Actix Web
- Serde
- Tokio
- HMAC + SHA256 for Telegram auth verification

### Database
- PostgreSQL
- SeaORM added in dependencies as the ORM direction for entities and migrations

Current Rust dependencies already include `actix-web`, `actix-files`, `sea-orm`, `tokio`, `serde`, `chrono`, `uuid`, `dotenvy`, `tracing`, `hmac`, `sha2`, and `hex`. :contentReference[oaicite:1]{index=1}

### Frontend
- Static HTML pages served by Rust
- Telegram WebApp SDK
- Telegram Login Widget / OAuth iframe fallback
- TON Connect UI

---

## Current runtime architecture

### Development runtime
In development, the app runs through Docker Compose with these services:

- `app` — Rust application container
- `db` — PostgreSQL 16
- `pgadmin` — optional PostgreSQL UI
- `ngrok` — public HTTPS tunnel for Telegram / web testing

Development compose currently maps:

- app: `8080:8080`
- db: `5432:5432`
- pgAdmin: `5050:80`

and uses:

- `DATABASE_URL=postgres://app_user:dev_password@db:5432/app_db`

This is defined in `docker-compose.dev.yml`. :contentReference[oaicite:2]{index=2}

### Production runtime
Production uses:

- Rust multi-stage Docker build
- PostgreSQL
- container healthchecks
- resource limits for app and DB

This is defined in `docker-compose.prod.yml` and `Dockerfile.prod`. :contentReference[oaicite:3]{index=3}

---

## Docker behavior right now

### `Dockerfile.dev`
Development image uses:

- `rustlang/rust:nightly-slim`
- `watchexec`
- mounted source code
- automatic restart on Rust / migration / Cargo changes

The dev container runs:

- `cargo run`

through `watchexec`, watching:

- `src`
- `migrations`
- `Cargo.toml`

This is the current hot-reload style dev workflow. :contentReference[oaicite:4]{index=4}

### `Dockerfile.prod`
Production build uses:

1. builder stage with Rust
2. final runtime stage with Debian slim
3. compiled binary `expert-hub`
4. `/health` healthcheck endpoint expected on port `8080`

This is already wired in the prod image. :contentReference[oaicite:5]{index=5}

---

## Current backend behavior

Current backend entry point is `src/main.rs`.

Right now the backend does these things:

### 1. Serves static frontend files
The Rust server serves files from `./public` and uses `index.html` as the root file. :contentReference[oaicite:6]{index=6}

### 2. Verifies Telegram web login
Endpoint:

- `POST /tg-auth`

What it does now:

- accepts Telegram auth payload
- rebuilds Telegram `data_check_string`
- computes HMAC-SHA256 using bot token derived secret
- checks received hash
- checks auth freshness
- returns verified Telegram profile JSON on success

This is already implemented in `src/main.rs`. :contentReference[oaicite:7]{index=7}

### 3. Accepts wallet link requests
Endpoint:

- `POST /link-wallet`

What it does now:

- accepts TON wallet address
- accepts chain
- accepts optional `telegram_id`
- currently logs the mapping request
- database persistence is still TODO

This is also already implemented in `src/main.rs`. :contentReference[oaicite:8]{index=8}

---

## Current frontend pages

### `public/index.html`
This is the current marketplace shell main page.

Its role right now:

- show a simple marketplace-style landing page
- show search mock
- show category chips
- link user into expert onboarding
- in Mini App context, this page is the place where Telegram app-context user detection works and can be used before moving to later pages

### `public/expert-new.html`
This is the current expert onboarding page.

Its role right now:

- show Telegram identity block
- show TON wallet block
- show next onboarding steps
- support both Telegram Mini App context and normal web context
- show connected Telegram user when app context is passed correctly
- show Telegram web login fallback when running outside Mini App context

---

## Current identity flow

There are currently **two identity contexts** in the project.

### 1. Telegram Mini App context
When the app is opened inside Telegram Mini App context, the project uses Telegram WebApp user data as the primary identity source.

Important practical detail:

- the user is detected on the main page in Mini App context
- that detected identity can then be passed forward to the expert onboarding page

This is the direction that solved the app-context detection problem during the current stage of work.

### 2. Web browser context
When the page is opened outside Telegram app context, the project falls back to Telegram web login.

That web auth flow currently works through Telegram OAuth widget / iframe and then calls:

- `POST /tg-auth`

for backend verification.

---

## Current TON wallet flow

TON wallet is already connected on the frontend through TON Connect UI.

Current behavior:

- user can connect wallet in frontend
- frontend reads wallet address and chain
- frontend sends them to backend through `/link-wallet`
- backend currently only accepts and logs the mapping
- real DB persistence is not implemented yet

The external TON manifest used by the project is documented separately. :contentReference[oaicite:9]{index=9}

---

## Current database state

The real full database implementation is **not finished yet**.

What exists right now:

- PostgreSQL runtime
- DB initialization scripts
- SeaORM added to Rust dependencies
- MVP schema draft documented separately

Current init scripts enable:

- `uuid-ossp`
- `pg_stat_statements`

This is already present in both init SQL files. :contentReference[oaicite:10]{index=10}

### Current documented MVP schema
The current schema draft defines these main tables:

- `experts`
- `tags`
- `categories`
- `reviews`
- `calendar`
- `bookings`

It also already defines expert fields such as:

- Telegram identity
- wallet address
- rating
- review count
- timezone
- working days
- work start / end time
- allowed session durations
- notice and buffer settings

and explains that many-to-many relations should be implemented with join tables like:

- `expert_tags`
- `expert_categories`

This is documented in the current database draft. :contentReference[oaicite:11]{index=11}

---

## What is already connected

Right now these parts are already connected together:

- Rust server serves static frontend
- frontend can run in browser and Telegram context
- Telegram web auth can be verified by backend
- TON wallet can be connected in frontend
- frontend can send wallet data to backend
- PostgreSQL runs in Docker
- ORM direction is chosen: SeaORM
- entity structure is already drafted at documentation level

---

## What is not finished yet

At the current stage, these parts are still not implemented fully:

- real SeaORM entities in code
- migrations crate and migration files
- DB persistence for Telegram users
- DB persistence for wallet links
- DB persistence for experts / tags / categories / reviews / bookings
- calendar provider integration
- public expert page
- booking confirmation flow
- TON payment / contract logic
- review flow

---

## Related project documentation

Current supporting documents in project documentation:

- `Database Structure - ExpertHub.txt` — current MVP schema draft for entities and relations. :contentReference[oaicite:12]{index=12}
- `Plan to MVP ExpertHub.txt` — business flow and MVP implementation direction. :contentReference[oaicite:13]{index=13}
- `ExpertHub-manifests.txt` — TON manifest reference. :contentReference[oaicite:14]{index=14}
- `expert-hub 17.03.2026.txt` — earlier exported snapshot of project structure, Docker files, backend entry point, and auth flow. :contentReference[oaicite:15]{index=15}

---

## Practical summary

Current Expert Hub is a working Rust + Docker + PostgreSQL project shell with:

- Telegram app/web identity handling
- TON wallet UI connection
- marketplace main page
- expert onboarding page
- backend auth verification endpoint
- backend wallet-link endpoint
- documented MVP database design

It is not yet the full marketplace, but the base runtime, auth flow, wallet flow, and DB direction are already established.