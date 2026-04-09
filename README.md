# Expert Hub

Expert Hub is a Telegram-first expert marketplace MVP built with Rust, Actix Web, PostgreSQL, SeaORM, static frontend pages, Telegram web auth / Mini App context, TON wallet connection, and Google Calendar OAuth integration.

The project is currently focused on **Phase 1: expert onboarding, public expert pages, and real availability display**.

At this stage, the project already has:

- Rust backend with `actix-web`
- PostgreSQL in Docker
- SeaORM entities and migrations
- health endpoint
- Telegram web auth verification endpoint
- Telegram Mini App integration on frontend
- shared Telegram identity resolver for Mini App + stored web auth fallback
- TON wallet connection on frontend
- marketplace shell page
- expert onboarding page
- expert setup registration backend flow
- public slug support for experts
- profile-created success page
- expert public page
- expert edit page
- database tables for experts, calendar connections, bookings, payments, reviews, tags, categories, and sync events
- environment split for dev and prod Telegram bot auth
- Google OAuth credentials wired into runtime flow
- working Google OAuth callback flow
- frontend Google Calendar connection UI
- selection of up to 2 calendars per Google connection
- support for adding up to 5 calendar connection blocks in expert onboarding
- real public availability generation from expert schedule settings
- Google Calendar free/busy availability checks
- Google access token refresh during availability fetch
- Telegram Mini App deep-link routing using `startapp`

---

## Current project stage

The app is **not a full marketplace yet**.

Right now the real implemented direction is:

1. expert opens onboarding flow
2. connects Telegram
3. connects TON wallet
4. fills expert profile fields
5. connects Google Calendar
6. selects calendars from Google account
7. optionally adds more calendar connection blocks
8. submits expert setup to backend
9. backend creates or updates expert and related calendar connection rows
10. backend generates and serves expert public/edit page routes based on `public_slug`
11. public expert page loads real expert data
12. public expert page calculates real free slots for the next 7-day period
13. user can open the app through Telegram Mini App deep links:
  - `startapp=s` → register
  - `startapp={slug}` → expert public page

This means the project already has the backend and DB foundation for expert records and related structures, a working Phase 1 onboarding flow, and a real public availability view. Booking request flow, payment finalization, internal booking holds, session detection, and full review flow are still ahead.

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
- Chrono-TZ
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

This is currently the main Mini App entry page and the placeholder marketplace shell. It also acts as the Telegram Mini App launch router:
- `startapp=s` redirects to expert registration
- `startapp={slug}` redirects to the expert public page

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

- Telegram Mini App public expert link
- edit page link
- next-step hints for the expert

### `/e/{slug}`
Public expert page route.

This page currently handles:

- loading public expert profile data
- showing hourly rate
- showing allowed consultation durations
- duration picker UI
- real 7-day availability generation
- previous / next 7-day navigation
- owner-gated edit icon when Telegram identity resolves and matches the expert

### `/e/{slug}/edit`
Expert edit page route.

This is the private editing page shell for a specific expert slug.

It currently supports:

- Telegram identity resolution
- loading editable expert data by slug
- editing profile fields
- editing rate / schedule / durations / visibility
- viewing connected calendars
- choosing the primary calendar
- connecting additional Google calendars
- saving updated expert profile settings

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

### Expert API routes
The project now has working service-level expert API flow around:

- public expert lookup by slug
- edit expert lookup by slug
- expert profile update by slug
- public availability generation by slug and duration

Important public API route:

- `GET /api/experts/{slug}/public?offset_days={n}&duration_minutes={m}`

This route now returns:

- public expert profile data
- real availability for the requested 7-day period
- slots filtered by selected duration

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
If the page is opened in a real Telegram Mini App launch context, frontend can read Telegram user from:

`window.Telegram.WebApp.initDataUnsafe.user`

### 2. Web / stored fallback context
If Mini App user data is unavailable, frontend can fall back to a stored Telegram user resolved from earlier auth flow.

Shared Telegram auth logic now includes:

- Mini App user resolver
- stored Telegram user resolver
- shared `resolveTelegramUser()` helper
- Telegram WebApp init helper

This means owner-gated UI can work in both Mini App and previously authenticated web contexts, but only if Telegram identity is available through one of those paths.

### Telegram Mini App deep links
The app now supports short Telegram launch links through `startapp`:

- `https://t.me/expert_hub_bot?startapp=s` → registration
- `https://t.me/expert_hub_bot?startapp={slug}` → public expert page

The Mini App root page reads the launch parameter and internally redirects to the correct route.

### Dev / prod bot split
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
- wallet can also be updated from the edit page flow

At this stage wallet connection is still onboarding / profile-level only. Real payment flow and smart contract settlement logic are planned later.

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

### Current registration status
Compared to earlier state, calendar connection persistence is now much more real in the working flow:

- selected Google calendars are saved
- provider-backed connection labels are saved
- selected calendar names and timezones are saved
- access token / refresh token can be used for real free/busy checks
- connection rows can be marked as connected
- public availability can use saved Google connections directly

### Still not finished
There are still important limitations:

- Google OAuth session storage is still temporary backend runtime state
- on backend restart, temporary Google OAuth session ids are lost
- reconnect / reauth handling can still be improved
- durable sync state and background calendar sync are not implemented yet
- the registration/edit mapping still needs cleanup and hardening for production reliability

---

## Calendar status

Calendar integration is now **real enough to power public availability**, but still not production-complete.

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
- public availability fetch now uses real Google free/busy API
- access token refresh is handled during availability fetch
- enabled Google calendars are used as blockers for slot calculation

### Still not finished
- Google OAuth sessions are still temporary backend runtime state
- there is no durable long-term background sync process yet
- `calendar_sync_events` are not yet powering a real sync pipeline
- Calendly is still placeholder only
- connected calendar editing UX is still basic
- calendar picker still uses a browser prompt instead of a custom modal/list UI
- there is no advanced cache / sync optimization layer yet

So calendar support is now **real for onboarding and public availability**, but not yet production-complete.

---

## Public expert page status

Public slug support exists and the public expert page is now real enough to demonstrate the expert-facing marketplace flow.

Current public page behavior:

- show public expert data
- show hourly rate
- show allowed consultation durations
- show a duration picker
- default to the lowest allowed duration
- show real free slots for the selected duration
- page through previous / next 7-day windows
- do not expose names/details of busy calendar events
- keep personal event data private
- show owner-gated edit icon when current Telegram identity matches the expert

### Availability logic currently implemented
Public availability now applies:

- working days
- work start / end time
- selected duration
- minimum notice
- max days ahead
- buffer before / after
- Google busy intervals
- internal booking blockers

This means slot generation is no longer fake placeholder data.

### Current pricing display direction
The expert stores an `hourly_rate`.

Public page direction is now:

- show base pricing as hourly rate
- derive session price from selected duration later in booking / summary flow

Example:
- 30 min = half hourly rate
- 60 min = full hourly rate

---

## Edit expert page status

Edit page is now beyond a shell and is a real profile-management page, though still incomplete.

Current implemented direction:

- support Telegram identity resolution / connect flow
- load expert data for the slug
- compare current Telegram identity to expert owner identity
- allow the owner to edit expert-controlled fields
- save profile updates back to backend
- show existing connected calendars
- allow primary calendar selection
- support adding more Google calendar connections

### Fields currently meaningful on edit page
Main expert-facing fields:

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
- wallet replacement flow

### Fields intentionally not for direct editing right now
These should stay backend-controlled or hidden for now:

- timezone
  source of truth should come from calendar
- calendar conflict mode
- booking target strategy
- tokens / secrets / raw provider metadata
- review counters / rating snapshots
- system sync fields

### Important note
Ownership / mismatch handling still needs final hardening and redirect behavior cleanup. The intended direction is clear, but this is still not a finished production auth-guard flow.

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

### Public expert page
- `public/js/expert-public.js`

### Expert onboarding
- `public/js/expert-new/dom.js`
- `public/js/expert-new/expert-draft.js`
- `public/js/expert-new/calendar-draft.js`
- `public/js/expert-new/ui.js`
- `public/js/expert-new/calendar.js`
- `public/js/expert-new/modals.js`
- `public/js/expert-new/register.js`
- `public/js/expert-new/index.js`

### Expert edit page
- `public/js/expert-edit/expert-edit.js`

This refactor direction is correct and should continue.

---

## What is working now

At this moment, the project can already demonstrate this realistic flow:

1. expert opens the app
2. connects Telegram
3. connects TON wallet
4. fills schedule / rate / profile
5. connects Google Calendar
6. selects calendars
7. submits setup
8. receives a Telegram Mini App public profile link
9. opens public expert page
10. sees real availability for selected duration
11. owner can reach the edit page from owner-gated controls

This is already much stronger than an HTML shell MVP.

---

## What is still ahead

Major next steps still ahead of the current implementation:

### Booking flow
- slot click → booking intent
- booking request record creation
- booking confirmation UI
- quote calculation from hourly rate and duration
- expert approval / rejection flow

### Payment / TON contract flow
- real payment creation
- TON contract integration
- payment locking
- settlement / refund rules

### Session / consultation lifecycle
- internal booking holds
- confirmed booking state transitions
- session outcome tracking
- Telegram call detection integration
- no-show handling

### Review flow
- review creation after consultation
- review visibility rules
- system-generated no-show tags
- rating recalculation

### Calendar hardening
- durable provider token persistence hardening
- reconnect handling
- background sync strategy
- proper use of `calendar_sync_events`
- better multi-calendar UX

### Auth hardening
- final owner mismatch redirect logic
- stricter edit-page owner enforcement
- clearer Telegram auth prompts for protected actions

### Marketplace layer
- real expert list
- category browsing
- search
- featured experts
- ranking / discovery

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
- tunnel path forwarding expectation
- static files catching route before dynamic handler

---

## Local development

Typical local startup:

```bash
docker compose -f docker-compose.dev.yml --env-file .env.local up --build