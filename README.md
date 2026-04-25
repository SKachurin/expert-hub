# Expert Hub

Expert Hub is a Telegram-first expert marketplace MVP built with Rust, Actix Web, PostgreSQL, SeaORM, static frontend pages, Telegram Mini App auth, TON wallet connection, Google Calendar OAuth, real availability generation, and a separate internal TON Worker for booking escrow contract preparation.

## Table of contents

- [Current status](#current-status)
- [Stack](#stack)
- [Main pages](#main-pages)
- [Current backend routes](#current-backend-routes)
- [Database](#database)
- [Booking statuses](#booking-statuses)
- [Telegram auth and navigation logic](#telegram-auth-and-navigation-logic)
- [Google Calendar logic](#google-calendar-logic)
- [TON payment architecture](#ton-payment-architecture)
- [Current TON booking/payment flow](#current-ton-bookingpayment-flow)
- [TON Worker payload contract](#ton-worker-payload-contract)
- [Current BookingEscrow contract logic](#current-bookingescrow-contract-logic)
- [Telegram research / session detection direction](#telegram-research--session-detection-direction)
- [Reviews](#reviews)
- [Frontend structure](#frontend-structure)
- [Dev / prod split](#dev--prod-split)
- [Deployment](#deployment)
- [Local development](#local-development)
- [What is working now](#what-is-working-now)
- [What is still ahead](#what-is-still-ahead)
- [Related service docs](#related-service-docs)

The current focus is:

1. expert onboarding
2. public expert pages
3. real availability from Google Calendar
4. booking request creation
5. TON escrow preparation
6. frontend TON Connect payment flow
7. later: expert confirmation, session detection, settlement, and reviews

The app is still not a full marketplace. It is now a working vertical slice around one public expert page and the beginning of the real booking/payment flow.

---

## Current status

At this stage the project already has:

- Rust backend with `actix-web`
- PostgreSQL in Docker
- SeaORM entities and migrations
- health endpoint
- Telegram Mini App integration
- Telegram web auth verification endpoint
- shared Telegram identity resolver for Mini App + stored fallback
- dev/prod Telegram bot split
- TON Connect UI on frontend
- separate TON Worker container for escrow contract preparation
- Google Calendar OAuth connection
- Google Calendar free/busy checks
- Google access-token refresh during availability fetch
- expert onboarding page
- expert edit page
- public expert page
- marketplace shell page
- popular experts list on homepage
- real public availability generation
- booking request creation from selected slot
- payment draft creation
- Rust → TON Worker prepare-booking integration
- frontend receives contract address, StateInit, and TON amount
- frontend attempts TON Connect transaction to deploy/fund the escrow contract

Currently still not finished:

- final successful wallet transaction approval is still being debugged
- payment funding verification is not implemented yet
- booking does not yet move to `funded` after on-chain confirmation
- expert Telegram confirmation flow is not implemented yet
- TON Worker contract actions are not yet wired into the full booking lifecycle
- Telegram call watcher / research service is not implemented yet
- review flow is not implemented yet
- production Mini App navigation still needs final hardening/caching checks

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

- Static HTML / CSS / JavaScript
- Telegram WebApp JS
- TON Connect UI

### Infra

- Docker / Docker Compose
- GitHub Actions deploy flow
- Nginx reverse proxy
- separate dev and prod environments
- reverse SSH tunnel in local dev for `dev.experthub.bar`

### Internal services

- TON Worker in a separate Docker container
- planned Telegram research / watcher service for controlled booking-call detection

---

## Main pages

### `/`

Marketplace shell page.

Current behavior:

- loads popular bookable experts from backend
- shows expert cards
- in normal browser context, expert cards link to Telegram Mini App deep links
- in Telegram Mini App context, expert cards should use internal `/e/{slug}` links
- acts as Mini App launch router for Telegram `startapp` params

Supported Telegram launch params:

```text
startapp=s           -> /expert-new.html
startapp=expert_new  -> /expert-new.html
startapp={slug}      -> /e/{slug}
````

Important: `expert_new` must not be treated as an expert slug.

### `/expert-new.html`

Expert onboarding page.

Current behavior:

* resolves Telegram identity
* supports Telegram Mini App context
* supports stored Telegram auth fallback
* connects TON wallet
* collects expert profile data
* collects rate, currency, working days, working hours, durations
* starts Google OAuth flow
* lets user select calendars
* supports multiple calendar connection blocks
* submits expert setup to backend
* backend creates or updates expert and calendar connection rows

### `/created.html?slug={public_slug}`

Success page after expert creation.

Current behavior:

* shows public expert link
* shows edit page link
* gives next-step hints

### `/e/{slug}`

Public expert page.

Current behavior:

* loads public expert data by slug
* shows expert name, username, bio, avatar, timezone
* shows hourly rate
* shows available session durations
* defaults to the lowest allowed duration
* calculates real availability for the selected duration
* uses Google Calendar busy blocks as blockers
* uses internal booking blockers
* lets customer select a slot
* shows derived session price from hourly rate and selected duration
* starts booking/payment flow
* owner sees edit button if Telegram identity matches expert owner

### `/e/{slug}/edit`

Expert edit page.

Current behavior:

* resolves Telegram identity
* loads editable expert data
* compares current Telegram identity with expert owner identity
* allows owner to update profile fields
* allows owner to update rate / schedule / duration / visibility fields
* shows connected calendars
* allows primary calendar selection
* supports adding another Google calendar connection
* supports TON wallet replacement flow

Still needs hardening:

* final owner mismatch behavior
* stricter edit protection
* cleaner unauthorized redirect / prompt

---

## Current backend routes

### Health

```http
GET /health
```

### Telegram auth

```http
POST /tg-auth
```

Verifies Telegram auth payload hash and returns verified Telegram user data.

### Wallet link

```http
POST /link-wallet
```

Currently mostly placeholder-style wallet linking endpoint.

### Expert setup

```http
POST /expert-setup/register
POST /experts/upsert
```

Used for onboarding and expert persistence.

### Google OAuth / calendar session

```http
GET  /oauth/google/start
GET  /oauth/google/callback
GET  /google/calendars/session/{session_id}
POST /google/calendars/session/{session_id}/select
```

These routes handle Google OAuth redirect, temporary OAuth session storage, Google calendar list loading, and calendar selection.

### Expert pages

```http
GET /e/{slug}
GET /e/{slug}/edit
```

Serve static page shells for public and edit pages.

### Expert API

```http
GET /api/experts/popular?limit={n}
GET /api/experts/{slug}/public?offset_days={n}&duration_minutes={m}
GET /experts/{slug}/edit-data
PATCH /experts/{slug}
DELETE /experts/{slug}/calendar-connections/{connection_id}
```

Current public API response includes:

* public expert profile
* real availability
* reviews array

### Booking API

Current booking/payment foundation exists.

Expected routes are handled through booking handlers and services.

Current flow:

1. frontend sends selected expert slug, selected slot, duration, Telegram user, and customer TON wallet
2. backend validates slot against real availability
3. backend creates booking with status `requested`
4. frontend calls begin-payment
5. backend moves booking to `awaiting_payment`
6. backend creates or updates payment row
7. backend converts USD quote to TON when needed
8. backend calls TON Worker `POST /contracts/prepare-booking`
9. backend stores returned contract address / transaction reference
10. backend returns payment payload to frontend
11. frontend calls TON Connect `sendTransaction()`

Current limitation:

* successful wallet approval and on-chain funding verification are not finished yet.

---

## Database

The current database foundation includes:

* `experts`
* `calendar_connections`
* `calendar_sync_events`
* `bookings`
* `payments`
* `reviews`
* `categories`
* `expert_categories`
* `tags`
* `expert_tags`
* `telegram_call_events`

### Important expert fields

The `experts` table stores:

* Telegram identity
* display name
* username
* bio
* photo URL
* TON wallet address
* timezone
* hourly rate
* currency
* working days
* work start time
* work end time
* allowed session durations
* minimum notice minutes
* buffer before minutes
* buffer after minutes
* max days ahead
* calendar conflict mode
* booking target strategy
* active flag
* bookable flag
* expert rating
* reviews count
* public slug

### Important booking fields

The `bookings` table stores:

* expert id
* optional calendar connection id
* requester Telegram identity
* requester TON wallet
* expert timezone
* requested duration
* hourly rate snapshot
* quoted amount
* currency
* slot start / end
* booking status
* expert confirmation / rejection timestamps
* payment deadline / hold expiry
* external calendar sync fields
* session tracking timestamps
* outcome source
* metadata JSON

### Important payment fields

The `payments` table stores:

* booking id
* expert id
* customer Telegram id
* amount
* currency
* status
* customer TON wallet
* expert TON wallet
* contract address
* transaction reference

---

## Booking statuses

Current intended statuses:

```text
requested
awaiting_payment
funded
waiting_for_session
in_grace_period
completed
expert_no_show
customer_no_show
refunded
review_open
closed
```

Current implementation already uses the early booking/payment statuses:

```text
requested
awaiting_payment
```

Current blocker statuses for availability:

```text
requested
awaiting_payment
funded
waiting_for_session
in_grace_period
```

Meaning:

* `requested` — booking row created from selected slot
* `awaiting_payment` — customer has started payment flow
* `funded` — contract/payment is funded
* `waiting_for_session` — booking is funded and waiting for scheduled time
* `in_grace_period` — expert is present, waiting for customer
* `completed` — consultation connection detected
* `expert_no_show` — expert failed to connect
* `customer_no_show` — customer failed to connect
* `refunded` — contract returned money to customer
* `review_open` — customer can leave review
* `closed` — review submitted or review window expired

---

## Telegram auth and navigation logic

The project supports two frontend contexts.

### 1. Telegram Mini App context

Frontend can read Telegram user from:

```js
window.Telegram.WebApp.initDataUnsafe.user
```

### 2. Web / stored fallback context

If Mini App user data is unavailable, frontend can fall back to stored Telegram user data from earlier auth flow.

Shared auth lives in:

```text
public/js/shared/telegram-auth.js
```

Shared app config lives in:

```text
public/js/shared/app-config.js
```

Current bot split:

```text
dev bot:  @expert_hub_bot
prod bot: @experthub_bbot
```

The Mini App root page must route Telegram `startapp` params correctly:

```text
s          -> /expert-new.html
expert_new -> /expert-new.html
{slug}     -> /e/{slug}
```

When already inside Telegram Mini App, expert card clicks should use internal app routes:

```text
/e/{slug}
```

When outside Telegram Mini App, public expert links should use Telegram deep links:

```text
https://t.me/{BOT}?startapp={slug}
```

---

## Google Calendar logic

Calendar integration is real enough to power public availability.

Already implemented:

* Google OAuth redirect
* Google account info loading
* Google calendar list loading
* calendar selection
* saving selected calendars into `calendar_connections`
* storing access token / refresh token
* using Google free/busy API during availability generation
* refreshing access token on expiry
* using enabled calendars as busy blockers

Still not finished:

* durable Google OAuth session storage
* durable background sync
* advanced reconnect / reauth flow
* custom calendar picker UI
* use of `calendar_sync_events` as a real sync pipeline
* Calendly integration

Availability source of truth:

```text
expert schedule settings
+ Google Calendar free/busy
+ internal active booking blockers
```

The app should not persist fake future slots as source of truth.

---

## TON payment architecture

The main Rust app must not contain TON SDK / Blueprint mechanics directly.

### Main Rust backend owns

* experts
* customers
* bookings
* payments
* Telegram identity
* Google Calendar availability
* session outcome decisions
* database state
* review flow
* when to ask TON Worker to prepare or execute contract actions

### TON Worker owns

* loading compiled `BookingEscrow` contract artifact
* preparing unique per-booking contract deployment data
* returning deterministic contract address
* returning serialized StateInit
* returning total amount the wallet must send
* sending controller/admin action messages to contracts when requested

### Current internal worker URL

```env
TON_WORKER_BASE_URL=http://ton-worker:8081
TON_WORKER_AUTH_TOKEN=local-dev-token
```

### Current TON Worker endpoints

```http
GET  /health
POST /contracts/prepare-booking
POST /contracts/{contract_address}/action
```

The worker must stay internal. It should not be called directly from the browser.

---

## Current TON booking/payment flow

Current implemented flow:

1. customer opens `/e/{slug}`
2. customer selects duration
3. customer selects available slot
4. customer connects TON wallet
5. frontend creates booking request
6. Rust backend validates selected slot
7. Rust backend creates `bookings` row with status `requested`
8. frontend begins payment
9. Rust backend verifies booking ownership by Telegram id
10. Rust backend updates booking to `awaiting_payment`
11. Rust backend creates or updates `payments` row with status `awaiting_payment`
12. if booking currency is `USD`, backend fetches TON/USD rate and converts quote to TON
13. backend builds TON Worker prepare payload
14. backend calls `POST /contracts/prepare-booking`
15. TON Worker prepares unique contract address and StateInit
16. backend stores returned contract address / transaction reference
17. backend returns payment data to frontend
18. frontend calls TON Connect `sendTransaction()`

Current frontend TON Connect transaction shape:

```js
await tonConnectUi.sendTransaction({
    validUntil: Math.floor(Date.now() / 1000) + 300,
    network: expectedChain,
    messages: [
        {
            address: result.contract_address,
            amount: result.amount_nano_ton,
            stateInit: result.state_init_boc
        }
    ]
});
```

When this succeeds, the customer wallet should deploy and fund the prepared contract.

Current issue:

* Telegram Wallet is still declining the transaction in the current test flow.
* The returned contract address is expected to be a new deterministic contract address for that booking.
* The app should send the transaction to that contract address with `stateInit`, not to the expert wallet directly.
* The exact wallet rejection reason still needs debugging.

---

## TON Worker payload contract

Current Rust → TON Worker prepare payload should use the worker’s expected request shape.

Current target shape:

```json
{
  "booking_id": 1,
  "payment_id": 1,
  "customer_telegram_id": 111111,
  "expert_telegram_id": 222222,
  "customer_wallet": "EQ...",
  "expert_wallet": "EQ...",
  "amount_nano_ton": "100000000",
  "slot_start_unix": 1760000000,
  "expert_confirmation_deadline_unix": 1760086400,
  "session_outcome_deadline_unix": 1760090000
}
```

Important:

* Do not send old mixed fields if the worker currently expects `amount_nano_ton`.
* If the worker schema expects `amount` + `currency`, then Rust DTO and worker DTO must be changed together.
* Rust and worker DTOs must always match exactly.

Current response shape:

```json
{
  "contract_address": "EQ...",
  "state_init_boc": "te6cck...",
  "amount_nano_ton": "250000000",
  "recommended_gas_buffer_nano_ton": "150000000",
  "total_deploy_value_nano_ton": "250000000"
}
```

Meaning:

* `contract_address` — deterministic address of the future booking escrow contract
* `state_init_boc` — serialized TON StateInit
* `amount_nano_ton` — total amount customer wallet should send
* `recommended_gas_buffer_nano_ton` — deploy/gas buffer included in total
* `total_deploy_value_nano_ton` — explicit total for readability

---

## Current BookingEscrow contract logic

The current escrow contract is per booking.

Stored data:

```text
amountNanoTon

state
fundedAtUnix
finalizedAtUnix

customerRatingForExpert
expertRatingForCustomer

parties:
  customerWallet
  expertWallet
  controllerWallet

meta:
  bookingId
  expertTelegramId
  customerTelegramId
  slotStartUnix
  expertConfirmationDeadlineUnix
  sessionOutcomeDeadlineUnix
```

Contract states:

```text
STATE_AWAITING_FUNDING = 0
STATE_FUNDED_WAITING_EXPERT = 1
STATE_WAITING_SESSION = 2
STATE_PAID_TO_EXPERT = 3
STATE_REFUNDED_TO_CUSTOMER = 4
```

Contract actions:

```text
expert_confirm
expert_decline
session_connected
customer_no_show
expert_no_show
set_customer_rating
set_expert_rating
```

Payout rules:

```text
expert_decline    -> refund customer
expert_no_show    -> refund customer
session_connected -> pay expert
customer_no_show  -> pay expert
```

The TON Worker does not decide these outcomes. The backend decides the outcome and instructs the TON Worker.

---

## Telegram research / session detection direction

The booking/session completion logic is planned around a Telegram research service.

Source of truth:

```text
Telegram research service event
```

Manual claims must not override system events.

Current planned V1 direction:

* use a controlled Telegram conference/group call per booking
* assign a Telethon watcher account to the booking
* watcher joins as silent intermediary
* watcher listens for Telegram MTProto participant updates
* watcher detects expert/customer participation
* watcher sends system event to Expert Hub
* Expert Hub stores raw event in `telegram_call_events`
* Expert Hub updates booking/payment outcome
* Expert Hub calls TON Worker action if needed

Intended outcome logic:

```text
both expected users detected -> completed
expert absent                -> expert_no_show -> refund customer
customer absent after grace  -> customer_no_show -> pay expert
```

Still pending:

* build watcher service
* test exact Telegram MTProto event flow
* define final event names
* connect watcher events to booking status transitions
* connect final booking outcomes to TON Worker contract actions

---

## Reviews

Reviews are planned but not finished.

Planned rules:

* one booking = one customer review
* reviews open after `completed`, `expert_no_show`, or `customer_no_show`
* system no-show tags are attached automatically
* system tags cannot be removed manually
* expert rating and review count are recalculated after review submission

---

## Frontend structure

Shared:

```text
public/js/shared/app-config.js
public/js/shared/dom-utils.js
public/js/shared/telegram-auth.js
```

Index:

```text
public/js/index.js
```

Created page:

```text
public/js/created.js
```

Public expert page:

```text
public/js/expert-public.js
```

Expert onboarding:

```text
public/js/expert-new/
```

Expert edit:

```text
public/js/expert-edit/
```

Current important frontend logic:

* `index.js` routes Telegram `startapp` params
* `index.js` builds internal links inside Mini App and Telegram deep links outside Mini App
* `expert-public.js` loads public expert profile and availability
* `expert-public.js` handles slot selection, booking request, payment preparation, and TON Connect transaction
* `app-config.js` controls dev/prod bot and TON network split

---

## Dev / prod split

Dev domain:

```text
dev.experthub.bar
```

Production domain:

```text
experthub.bar
```

Dev bot:

```text
@expert_hub_bot
```

Prod bot:

```text
@experthub_bbot
```

TON network split:

```text
dev  -> TESTNET
prod -> MAINNET
```

Important:

* bot username and bot token must match the environment
* Telegram auth hash verification depends on the correct bot token
* Mini App links must use the correct bot for the domain
* frontend cache-busting query strings must be bumped after JS changes

---

## Deployment

Production uses:

```text
docker-compose.prod.yml
Dockerfile.prod
GitHub Actions deploy flow
Nginx reverse proxy
```

Current useful checks:

```bash
docker ps
docker logs -f expert-hub-app-1
curl -i https://experthub.bar/
curl -i https://experthub.bar/e/sergei-rz
curl -i 'https://experthub.bar/api/experts/sergei-rz/public?offset_days=0'
curl -s https://experthub.bar/js/index.js | grep -n "startapp\|BOT\|public_slug\|href\|/e/"
curl -s https://experthub.bar/js/shared/app-config.js
```

If `/api/experts/popular` works but clicking the expert does not load the public page, check:

* `public/js/index.js`
* `public/js/shared/app-config.js`
* Telegram `startapp` routing
* cache-busting query string in `public/index.html`
* whether `expert_new` is accidentally treated as a slug
* whether Mini App context is detected too strictly

---

## Local development

Typical local startup:

```bash
docker compose -f docker-compose.dev.yml --env-file .env.local up --build
```

Typical prod startup:

```bash
docker compose -f docker-compose.prod.yml --env-file .env up -d --build
```

Run app logs:

```bash
docker logs -f expert-hub-app-1
```

Run TON Worker logs:

```bash
docker logs -f ton-worker-experthub
```

---

## What is working now

The project can currently demonstrate:

1. expert opens onboarding
2. expert connects Telegram
3. expert connects TON wallet
4. expert fills profile, rate, schedule, durations
5. expert connects Google Calendar
6. expert selects calendars
7. backend saves expert and calendar connections
8. public expert page is generated by slug
9. public page loads expert data
10. public page shows real availability
11. customer selects slot
12. backend creates booking request
13. backend creates payment draft
14. backend calls TON Worker
15. TON Worker returns deterministic escrow contract address and StateInit
16. frontend attempts TON Connect escrow deployment/funding transaction

---

## What is still ahead

Next priorities:

1. Fix Telegram Wallet transaction rejection.
2. Confirm correct contract address / StateInit / amount format.
3. Add payment funding verification after successful wallet approval.
4. Move payment to `funded`.
5. Move booking to `funded` / `waiting_for_session`.
6. Add expert confirmation flow through Telegram bot.
7. Wire `expert_confirm` and `expert_decline` contract actions.
8. Add Telegram watcher service.
9. Store watcher events in `telegram_call_events`.
10. Apply final session outcomes.
11. Wire `session_connected`, `customer_no_show`, and `expert_no_show` contract actions.
12. Add review flow.
13. Add marketplace discovery later.

---

## Related service docs

Detailed TON Worker implementation notes belong in the separate worker repository README:

```text
ton-worker-experthub/README.md
```

The main Expert Hub README should describe only how the Rust backend integrates with the worker.
