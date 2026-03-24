# Expert Hub

## Current stage

Expert Hub is currently a Rust-based, Telegram-first MVP backend and frontend foundation for an expert marketplace.

At this stage, the project already has:

- Rust backend with `actix-web`
- PostgreSQL in Docker
- SeaORM entities and migrations wired into the project
- Telegram web auth fallback for normal browser context
- Telegram Mini App context support
- TON wallet connection on the frontend
- marketplace main page
- expert onboarding page
- working onboarding submit flow from both web and Telegram application context
- composite onboarding request handled by backend and split into expert + calendar persistence
- working MVP database schema created through migrations

The current codebase is not yet a full booking platform, but it is no longer only a schema draft.

Right now it is mainly:

1. Rust backend foundation
2. Docker runtime
3. frontend host for web and Telegram app context
4. Telegram identity handling
5. TON wallet connection handling
6. expert onboarding flow with DB persistence
7. real database schema and entity layer for MVP tables

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
- `src/config.rs`
- `src/db.rs`
- `src/state.rs`
- `src/http/routes.rs`
- `src/http/handlers/auth.rs`
- `src/http/handlers/health.rs`
- `src/http/handlers/expert_setup.rs`
- `src/entities/`
- `src/services/experts.rs`
- `src/services/calendar_connections.rs`
- `src/services/expert_setup.rs`
- `migrations/`
- `public/index.html`
- `public/expert-new.html`
- `scripts/init-dev.sql`
- `scripts/init-prod.sql`

---

## Tech stack currently used

### Backend
- Rust
- Actix Web
- Serde
- Tokio
- SeaORM
- HMAC + SHA256 for Telegram auth verification

### Database
- PostgreSQL
- SeaORM entities
- SeaORM migrations

### Frontend
- Static HTML pages served by Rust
- Telegram WebApp SDK
- Telegram Login Widget / OAuth iframe fallback
- TON Connect UI
- local browser draft storage for onboarding form state

---

## Current runtime architecture

### Development runtime
In development, the app runs through Docker Compose with these services:

- `app` — Rust application container
- `db` — PostgreSQL 16
- `pgadmin` — optional PostgreSQL UI
- `ngrok` — public HTTPS tunnel for Telegram / web testing

### Production runtime
Production uses:

- Rust multi-stage Docker build
- PostgreSQL
- container healthchecks
- resource limits for app and DB

---

## Current database state

The project now has a real MVP database layer in code.

What exists right now:

- PostgreSQL runtime
- DB initialization scripts
- SeaORM entity files
- SeaORM migrations crate
- real migration files for MVP schema
- successful migration path for the current schema

Current init scripts enable:

- `uuid-ossp`
- `pg_stat_statements`

### Current implemented MVP tables

The project now includes real entities and migrations for:

- `calendar_connections`
- `experts`
- `tags`
- `categories`
- `reviews`
- `bookings`
- `expert_tags`
- `expert_categories`
- `payments`
- `telegram_call_events`

### Current booking statuses

The booking flow is aligned to the MVP plan and uses these statuses:

- `requested`
- `awaiting_payment`
- `funded`
- `waiting_for_session`
- `in_grace_period`
- `completed`
- `expert_no_show`
- `customer_no_show`
- `refunded`
- `review_open`
- `closed`

### Current payments and session outcome support

The schema now also includes:

- `payments` for booking-linked payment records
- `telegram_call_events` for raw research-service session outcome data

This is the base needed for:

- booking persistence
- TON payment tracking
- system-driven no-show/completion decisions

---

## What is already connected

Right now these parts are already connected together:

- Rust server serves static frontend
- frontend can run in browser and Telegram context
- Telegram web auth can be verified by backend
- Telegram Mini App identity can be detected in app context
- TON wallet can be connected in frontend
- PostgreSQL runs in Docker
- SeaORM is integrated
- MVP entities exist in code
- MVP migrations exist in code
- current DB schema can be created successfully

In addition, the onboarding flow now works end-to-end:

- user can open expert onboarding page in web or Telegram context
- Telegram identity is shown on screen
- TON wallet can be connected on screen
- expert profile fields can be filled on screen
- form draft is restored from browser storage on reload
- final onboarding submit creates:
    - one row in `experts`
    - one row in `calendar_connections`
- this flow has been tested successfully from both:
    - normal web context
    - Telegram application / Mini App context

---

## Current onboarding flow

The current onboarding page is no longer just a visual draft.

It now includes these sections in one screen:

1. Telegram identity
2. TON wallet
3. Expert profile
4. Calendar
5. Progress / readiness tracking
6. Final submit button

### Current expert profile fields on screen

The onboarding page currently collects:

- display name
- description
- timezone
- hourly rate
- currency
- working days
- work start time
- work end time
- allowed session durations

### Current calendar section

The onboarding screen also includes a calendar section with:

- provider selector
- connect action
- calendar readiness state

At the current stage, this is still a placeholder onboarding connection step, not a real OAuth / API integration yet.

### Current submit behavior

The final button submits one onboarding request from the frontend.

On the backend, that request is handled as one screen-level payload and then split internally into:

- expert persistence
- calendar connection persistence

This means the current onboarding flow already writes normalized data into multiple tables while keeping one final registration action in the UI.

---

## What is not finished yet

At the current stage, these parts are still not implemented fully:

- real Google Calendar integration
- real Calendly integration
- proper calendar link / OAuth / API connection flow
- expert personal edit page after registration
- support for editing remaining expert scheduling fields not yet exposed in onboarding
- public expert page backed fully by DB data
- real persistence flow for booking creation
- real persistence flow for payment creation and updates
- booking confirmation flow
- TON payment / contract business logic
- Telegram research-service integration into booking outcome flow
- review flow on top of booking outcomes

---

## Immediate next steps

### 1. Open the expert personal edit page after registration
After the user presses **Register me as an expert**, the next logical step is to redirect them into their personal expert page with all editable fields available.

Reason:

- the onboarding screen already covers the minimum required setup
- the expert row is now being created successfully
- some expert fields are still backend-defaulted and not yet editable from onboarding
- after registration, the user should land in a proper edit page to finish and improve their profile

This personal expert page should become the place where the user can later update:

- description
- rates
- work schedule
- durations
- tags
- categories
- calendar connection details
- future expert profile settings

### 2. Implement the real Google Calendar connection
The current calendar section is only a temporary MVP placeholder.

The next real integration step should be:

- implement actual Google Calendar connection
- store real calendar connection data instead of only placeholder provider/link state
- replace the current fake connect step with a real integration flow

This is the next backend + frontend integration priority after the expert registration flow.

---

## Related project documentation

Current supporting documents in project documentation:

- `Database Structure - ExpertHub.txt`
- `Plan to MVP ExpertHub.txt`
- `ExpertHub-manifests.txt`
- `expert-hub 17.03.2026.txt`
- `expert-hub 20.03.2026.txt`

---

## Practical summary

Current Expert Hub is a working Rust + Docker + PostgreSQL MVP foundation with:

- Telegram app/web identity handling
- TON wallet UI connection
- marketplace main page
- expert onboarding page
- backend auth verification endpoint
- composite onboarding submit flow
- real SeaORM entities
- real SeaORM migrations
- working database schema for the current MVP foundation
- successful expert registration persistence into normalized DB tables
- successful testing from both browser and Telegram application context

It is still not the full marketplace, but the project now already has:

- a real onboarding screen
- real onboarding persistence
- real database writes for expert registration
- real multi-context flow validation

The most logical next steps are:

1. open the user's personal expert edit page immediately after registration
2. implement the real Google Calendar connection
3. continue with booking creation on top of the now-working expert onboarding base
4. integrate Tera with Actix to handle your HTML templates