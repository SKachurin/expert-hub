# Expert Hub

Expert Hub is a Telegram-first expert marketplace MVP built with Rust, Actix Web, PostgreSQL, SeaORM, static frontend pages, Telegram web auth / Mini App context, TON wallet connection, and Google Calendar OAuth integration.

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
- Google OAuth credentials wired in env / deploy flow
- working Google OAuth callback flow
- frontend Google Calendar connection UI
- selection of up to 2 calendars per Google connection
- support for adding up to 5 calendar connection blocks in expert onboarding
- expert registration successfully creating expert records in DB

---

## Current project stage

The app is not a full marketplace yet.

Right now the real implemented focus is:

1. expert opens onboarding flow
2. connects Telegram
3. connects TON wallet
4. fills expert profile basics
5. connects Google Calendar
6. selects calendars from Google account
7. optionally adds more calendar connection blocks
8. submits expert setup to backend
9. backend creates or updates expert and related calendar connection records

This means the project already has the backend and DB foundation for expert records and related structures, plus a working Phase 1 onboarding flow.  
Real booking / payment / scheduling / availability sync logic is still incomplete.

---

## Stack

### Backend
- Rust
- Actix Web
- SeaORM
- PostgreSQL
- Serde
- Tokio
- Reqwest
- UUID
- Chrono

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
- Google Calendar OAuth start / callback continuation
- selecting up to 2 calendars from a Google account
- multiple calendar connection blocks
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

### Google OAuth / calendar session
- `GET /oauth/google/start`
- `GET /oauth/google/callback`
- `GET /google/calendars/session/{session_id}`
- `POST /google/calendars/session/{session_id}/select`

These routes handle Google OAuth redirect, temporary OAuth session storage, Google calendar list loading, and user calendar selection.

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
- TON wallet address
- one or more calendar connections
- selected calendars per Google connection

Backend registration flow creates or updates expert data and stores related calendar connection rows.

The registration flow is now working end-to-end at MVP onboarding level for expert creation.

### Important current backend detail
During expert registration, the backend currently creates `calendar_connections` rows, but these rows are still saved in a **minimal placeholder form**.

Right now the registration insert is saving mainly:

- `expert_id`
- `provider`
- `connection_label`
- `is_primary`
- `is_enabled`

As a result, many richer provider-related columns are still left empty at registration time, including fields such as:

- `account_email`
- `provider_account_id`
- `selected_calendar_id`
- `selected_calendar_name`
- `selected_calendar_timezone`
- `access_token`
- `refresh_token`
- `scopes_json`

Because of that, newly created Google calendar connection rows currently remain in a default / placeholder-style DB state rather than a fully hydrated provider-connected state.

---

## Calendar status

Calendar integration is now **partially real and working for onboarding**.

### Already done
- calendar connection entities and migrations were added
- backend service for calendar connection records exists
- Google OAuth env wiring was added into project/deploy flow
- real Google OAuth redirect flow is implemented
- Google account info can be loaded after OAuth
- Google calendar list can be loaded
- user can select up to 2 calendars from a connected Google account
- frontend can create multiple calendar connection blocks
- frontend allows up to 5 calendar blocks
- connected calendar names are shown in onboarding UI
- expert registration sends selected Google session references to backend
- expert registration creates corresponding `calendar_connections` rows in DB

### Still not finished
- Google OAuth sessions are currently temporary backend runtime state, not durable storage
- on backend restart, previously temporary Google OAuth session ids are lost
- frontend still needs a proper “revalidate connected calendar session on reload” flow
- backend registration does **not yet fully map Google session data into `calendar_connections`**
- `calendar_connections` rows are currently created with placeholder/minimal data and often remain in `pending` state
- real calendar sync / free-busy import is not implemented yet
- Calendly is still placeholder only
- connected calendar editing UX is still basic
- the calendar picker still uses a browser prompt, not a custom in-page modal/list UI

So calendar support is now **real for onboarding**, but not yet production-complete.

---

## Important current limitation

Google OAuth session data is currently stored temporarily in backend runtime memory before final expert registration.

That means:

- the frontend can keep local draft data
- but the actual Google access token / calendar session cannot live only in frontend storage
- if backend runtime memory is lost before final registration, the frontend can still show local draft state, but the Google session may no longer exist server-side

This is the main current limitation in calendar onboarding flow.

A second important limitation is that final registration currently creates DB rows for selected calendar connections, but does not yet persist the full Google provider/session metadata into those rows.

The next logical cleanup step is:

1. backend session revalidation / graceful invalidation on reload
2. full mapping of Google session data into `calendar_connections` during registration

---

## Google OAuth env variables

The project now expects Google OAuth credentials in runtime flow:

- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`
- `GOOGLE_REDIRECT_URI`

These are required for Google Calendar connection.

---

## Current frontend onboarding behavior

The onboarding UI currently includes:

- Telegram-required modal when user tries calendar connection without Telegram auth
- provider-required modal when user tries connecting calendar without choosing provider
- Google permission / OAuth error modal handling
- connected calendar names shown in the card after successful connection
- “Connect another calendar” block with hard cap of 5 total calendar blocks
- connected-state button styling for Google calendar editing / reconnect flow

The UI now reflects connected calendars much better than the earlier placeholder state.

---

## Current known gap between frontend and backend

Frontend currently shows connected Google calendars based on temporary selected session data and local draft state.

Backend registration currently uses that data enough to create expert and connection rows, but does not yet persist the full connected Google record into the `calendar_connections` table.

So the current system state is:

- onboarding UI works
- Google OAuth flow works
- calendar selection works
- expert creation works
- DB connection rows are created
- but saved connection records are still not fully provider-backed rows yet

This is the main next backend task.

---

## Local development

Typical local startup:

```bash
docker compose -f docker-compose.dev.yml --env-file .env.local up --build