# Expert Hub

Expert Hub is a Telegram-first expert marketplace MVP built with Rust, Actix Web, PostgreSQL, SeaORM, static frontend pages, Telegram web auth / Mini App context, TON wallet connection, and Google Calendar OAuth integration.

The project is currently focused on **Phase 1: expert onboarding and public expert pages**.

At this stage, the project already has:

- Rust backend with `actix-web`
- PostgreSQL in Docker
- SeaORM entities and migrations
- health endpoint
- Telegram web auth verification endpoint
- Telegram Mini App user detection on frontend
- TON wallet connection on frontend
- marketplace shell page
- expert onboarding page
- expert setup registration backend flow
- public slug support for experts
- profile-created success page
- expert public page shell
- expert edit page shell
- database tables for experts, calendar connections, bookings, payments, reviews, tags, categories, and sync events
- environment split for dev and prod Telegram bot auth
- Google OAuth credentials wired into runtime flow
- working Google OAuth callback flow
- frontend Google Calendar connection UI
- selection of up to 2 calendars per Google connection
- support for adding up to 5 calendar connection blocks in expert onboarding

---

## Current project stage

The app is **not a full marketplace yet**.

Right now the real implemented direction is:

1. expert opens onboarding flow
2. connects Telegram
3. connects TON wallet
4. fills basic expert profile fields
5. connects Google Calendar
6. selects calendars from Google account
7. optionally adds more calendar connection blocks
8. submits expert setup to backend
9. backend creates or updates expert and related calendar connection rows
10. backend generates and serves expert public/edit page routes based on `public_slug`

This means the project already has the backend and DB foundation for expert records and related structures, plus a working Phase 1 onboarding flow. Booking, payment finalization, real availability sync, session detection, and reviews are still incomplete.

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
- Rust Decimal

### Frontend
- Static HTML / CSS / JS
- Telegram WebApp JS
- TON Connect UI

### Infra
- Docker / Docker Compose
- GitHub Actions deploy flow
- separate dev and prod environments
- reverse SSH tunnel in local dev for `dev.experthub.bar`

---

## Current main pages

### `/`
Marketplace shell page.

This is currently a placeholder public homepage for the future marketplace. It explains the direction and links to expert onboarding.

### `/expert-new.html`
Main expert onboarding page.

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

### `/created.html?slug={public_slug}`
Success page shown after expert creation.

This page shows:

- public expert link
- edit page link
- next-step hints for the expert

### `/e/{slug}`
Public expert page route.

This is the public expert page shell for a direct expert URL.  
Frontend JS is expected to load public expert data and availability data from backend API routes.

### `/e/{slug}/edit`
Expert edit page route.

This is the private editing page shell for a specific expert slug.  
It is intended to become the full editor for expert-controlled profile fields.

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
- `POST /experts/upsert`

Expert creation / update flow used during onboarding and service-level expert persistence.

### Google OAuth / calendar session
- `GET /oauth/google/start`
- `GET /oauth/google/callback`
- `GET /google/calendars/session/{session_id}`
- `POST /google/calendars/session/{session_id}/select`

These routes handle Google OAuth redirect, temporary OAuth session storage, Google calendar list loading, and user calendar selection.

### Expert page / profile routes
- `GET /e/{slug}`
- `GET /e/{slug}/edit`

These routes return the static page shells for public and edit expert pages.

### Expert API direction
The project now also has expert-service work around:
- public expert lookup by slug
- edit expert lookup by slug
- expert profile update by slug

The exact final API shape is still being stabilized.

---

## Current database direction

The project already includes entities and migrations for the main marketplace foundation.

Important tables/entities currently present:

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

Not all of them are fully used yet in frontend flow, but the schema foundation is already in place for the marketplace MVP.

### Important expert fields already present in DB

The `experts` table already includes fields for:

- Telegram identity
- display name
- bio
- photo URL
- TON wallet address
- timezone
- hourly rate
- currency
- working days
- work start time
- work end time
- allowed session durations
- minimum notice minutes
- buffer before minutes
- buffer after minutes
- max days ahead
- calendar conflict mode
- booking target strategy
- active flag
- bookable flag
- expert rating
- reviews count
- public slug

### Important calendar connection fields already present in DB

The `calendar_connections` table already includes fields for:

- provider
- connection label
- primary / enabled flags
- connection status
- account email
- provider account id
- selected calendar id / name / timezone
- selected scheduling URL / event type fields
- access token / refresh token
- token expiry
- scopes JSON
- provider metadata
- sync cursor / last sync info
- public link

This means the DB is already wider than the current UI.

---

## Telegram auth logic

The project supports two contexts.

### 1. Telegram Mini App context
If the page is opened inside Telegram Mini App, frontend reads Telegram user from:

`window.Telegram.WebApp.initDataUnsafe.user`

### 2. Web auth fallback
If the page is opened in a normal browser, frontend uses Telegram OAuth widget / embed flow and then sends auth payload to backend `POST /tg-auth` for verification.

The project uses separate bot identities for dev and prod:

- dev bot: `@expert_hub_bot`
- prod bot: `@experthub_bbot`

This split is important because Telegram auth hash verification must use the matching bot token on backend.

---

## TON wallet status

Frontend already integrates TON Connect UI.

Current behavior:

- user can connect wallet on expert onboarding page
- connected wallet address is shown in UI
- wallet address is included in expert registration payload

At this stage wallet connection is onboarding-level only. Real payment flow and smart contract settlement logic are planned later.

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

The registration flow is working at MVP onboarding level for expert creation.

### Current known limitation in registration

During expert registration, backend creates `calendar_connections` rows, but those rows are still not fully hydrated with all provider-backed fields.

Right now the registration insert is still closer to a minimal placeholder form than a final synchronized provider record.

That means fields such as these may still remain empty or incomplete after onboarding:

- `account_email`
- `provider_account_id`
- `selected_calendar_id`
- `selected_calendar_name`
- `selected_calendar_timezone`
- `access_token`
- `refresh_token`
- `scopes_json`

So onboarding UI works, but calendar connection persistence is not yet fully finished.

---

## Calendar status

Calendar integration is now **partially real and working for onboarding**.

### Already done
- calendar connection entities and migrations exist
- Google OAuth env wiring was added
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
- Google OAuth sessions are still temporary backend runtime state, not durable storage
- on backend restart, previously temporary Google OAuth session ids are lost
- backend registration does not yet fully map Google session data into `calendar_connections`
- `calendar_connections` rows are still closer to minimal / pending rows than final provider-connected rows
- real calendar sync / free-busy import is not implemented yet
- Calendly is still placeholder only
- connected calendar editing UX is still basic
- calendar picker still uses a browser prompt instead of a custom in-page modal/list UI

So calendar support is now **real for onboarding**, but not yet production-complete.

---

## Public expert page status

Public slug support exists and public expert page HTML is now served.

Current direction of the public page:

- show public expert data
- show rates
- show allowed consultation durations
- later show free slots in a 7-day window
- later page through next 7-day windows
- do not expose names/details of busy calendar events
- keep personal event data private

### Important current dev note
The public page route is sensitive to route shape and static-serving order.  
The current route without trailing slash is the intended one:

- good: `/e/{slug}`
- bad in current dev setup: `/e/{slug}/`

If the browser requests the trailing-slash version, Nginx / static routing may return `404` before the dynamic route is reached.

---

## Edit expert page status

Edit page shell is now served.

Current intended direction:

- support the same Telegram identity detection / connect flow as onboarding
- if opened in Telegram, auto-resolve Telegram user
- if opened in web context, show Telegram connect flow
- after Telegram identity is known, load expert data for that slug
- show editable expert-controlled DB fields in grouped sections
- keep hidden/internal strategy fields controlled by backend defaults when not appropriate for UI

### Fields planned as editable on edit page
These are the main expert-facing fields worth editing:

- display name
- bio
- hourly rate
- currency
- working days
- work start time
- work end time
- allowed session durations
- minimum notice minutes
- buffer before minutes
- buffer after minutes
- max days ahead
- active flag
- available for calls / bookable flag
- primary calendar selection

### Fields intentionally not for direct editing right now
These should stay backend-controlled or hidden for now:

- timezone  
  source of truth should come from calendar
- calendar conflict mode
- booking target strategy  
  backend should keep working with fixed mode/default strategy unless UI is ready
- tokens / secrets / raw provider metadata
- review counters / rating snapshots
- system sync fields

---

## Frontend structure

The frontend was already being refactored away from giant inline page scripts.

Important current frontend structure includes:

### Shared
- `public/js/shared/app-config.js`
- `public/js/shared/dom-utils.js`
- `public/js/shared/telegram-auth.js`

### Index
- `public/js/index.js`

### Created page
- `public/js/created.js`

### Expert onboarding
- `public/js/expert-new/dom.js`
- `public/js/expert-new/expert-draft.js`
- `public/js/expert-new/calendar-draft.js`
- `public/js/expert-new/ui.js`
- `public/js/expert-new/calendar.js`
- `public/js/expert-new/modals.js`
- `public/js/expert-new/register.js`
- `public/js/expert-new/index.js`

This refactor direction is correct and should continue.

---

## Current development environment

### Dev app
The dev app runs locally on:

- `127.0.0.1:8080`

### Dev tunnel
The project uses a reverse SSH tunnel container so the remote dev domain can reach local app port:

- remote target: `root@108.181.246.49`
- reverse mapping: `18080 -> host.docker.internal:8080`

This is what makes `dev.experthub.bar` work against the local machine during dev.

### Important dev consequence
If a route works on:

`http://127.0.0.1:8080/...`

but not on:

`https://dev.experthub.bar/...`

then the problem is often outside Rust app code, usually one of:

- Nginx route handling
- trailing slash mismatch
- tunnel path forwarding expectation
- static files catching route before dynamic handler

---

## Local development

Typical local startup:

```bash
docker compose -f docker-compose.dev.yml --env-file .env.local up --build