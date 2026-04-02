# Expert Hub

Expert Hub is a Telegram-first expert marketplace MVP built with Rust, Actix Web, PostgreSQL, SeaORM, static frontend pages, Telegram web auth / Mini App context, and TON wallet connection.

The project is currently focused on **Phase 1: expert onboarding**.

At this stage, the system already has:

- Rust backend with `actix-web`
- PostgreSQL in Docker
- SeaORM entities and migrations
- health endpoint
- Telegram web auth verification endpoint
- Telegram Mini App user detection on frontend
- TON wallet connection on frontend
- expert onboarding page
- marketplace shell page
- expert setup registration backend flow
- database tables for experts, calendar connections, bookings, payments, reviews, tags, categories, and sync events
- environment split for dev/prod Telegram bot auth
- Google OAuth credentials wiring in env / deploy flow started

---

## Current project stage

The app is not a full marketplace yet.

Right now the real implemented focus is:

1. expert opens onboarding flow
2. connects Telegram
3. connects TON wallet
4. fills expert profile basics
5. chooses calendar provider
6. submits expert setup to backend

This means the project already has the backend and DB foundation for expert records and related structures, but the real booking / payment / scheduling flow is still incomplete.

---

## Stack

### Backend
- Rust
- Actix Web
- SeaORM
- PostgreSQL
- Serde
- Tokio

### Frontend
- Static HTML/CSS/JS
- Telegram WebApp JS
- TON Connect UI

### Infra
- Docker / Docker Compose
- GitHub Actions deploy flow
- separate dev and prod environments

---

## Main pages

### `/`
Marketplace shell page.

This is currently a placeholder public homepage for the future marketplace.  
It already explains the project direction and links to expert onboarding.

### `/expert-new.html`
Main expert onboarding page for Phase 1.

This page currently handles:

- Telegram identity detection
- Telegram web auth fallback
- TON wallet connection
- expert setup form
- local draft storage
- calendar provider selection
- expert registration request to backend

---

## Current backend routes

### Health
- `GET /health`

Basic health check used by deploy flow.

### Telegram auth
- `POST /tg-auth`

Verifies Telegram auth payload hash and returns verified Telegram user data.

### Wallet link
- `POST /link-wallet`

Currently placeholder-style endpoint for wallet linking flow.

### Expert setup
- `POST /expert-setup/register`

Creates or updates expert setup data and related calendar connection data.

---

## Current database direction

The project already includes entities and migrations for the main marketplace foundation.

Important tables/entities currently present in project work:

- `experts`
- `calendar_connections`
- `calendar_sync_events`
- `bookings`
- `payments`
- `reviews`
- `categories`
- `expert_categories`
- `tags`
- `expert_tags`
- `telegram_call_events`

Not all of them are fully used yet in frontend flow, but the schema foundation is already being built for the marketplace MVP.

---

## Telegram auth logic

The project supports two contexts:

### 1. Telegram Mini App context
If the page is opened inside Telegram Mini App, frontend reads Telegram user from `window.Telegram.WebApp.initDataUnsafe.user`.

### 2. Web auth fallback
If the page is opened in a normal browser, frontend uses Telegram OAuth widget / embed flow and then sends auth payload to backend `POST /tg-auth` for verification.

The project uses separate bot identities for dev and prod:

- dev bot: `@expert_hub_bot`
- prod bot: `@experthub_bbot`

This split is important because Telegram auth hash verification must use the matching bot token on backend.

---

## TON wallet

Frontend already integrates TON Connect UI.

Current behavior:

- user can connect wallet on expert onboarding page
- connected wallet address is shown in UI
- wallet address is included in expert registration payload

At this stage wallet connection is onboarding-level only.  
Real payment flow and smart contract flow are planned later.

---

## Expert setup registration

The current onboarding page collects and submits:

- Telegram identity
- display name
- bio
- timezone
- hourly rate
- currency
- working days
- work start time
- work end time
- allowed session durations
- selected calendar provider
- TON wallet address

Backend registration flow is handled through `expert_setup` service and `experts` service.

---

## Calendar status

Calendar integration is **in transition**.

### Already done
- calendar connection entities and migrations were added
- backend service for calendar connection records exists
- Google OAuth env wiring was added into project/deploy flow
- frontend structure for real Google calendar connection has started

### Not finished yet
- real end-to-end Google Calendar OAuth flow is not fully completed
- current codebase is between placeholder calendar state and real OAuth state
- selecting and persisting up to 2 Google calendars is the next active implementation step
- Calendly is still placeholder only

So calendar support is **partially wired, but not complete**.

---

## Google OAuth env variables

The project now expects Google OAuth credentials in runtime flow:

- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`
- `GOOGLE_REDIRECT_URI`

These were added into deploy/env flow because Google Calendar connection is being implemented.

---

## Local development

Typical local startup:

```bash
docker compose -f docker-compose.dev.yml --env-file .env.local up --build