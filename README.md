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

1. expert onboarding
2. public expert pages
3. real availability from Google Calendar
4. booking request creation
5. TON escrow preparation
6. frontend TON Connect escrow deployment/funding flow
7. backend on-chain funding verification through TON Worker
8. Telegram Bot expert confirmation flow
9. next: expert Confirm/Decline callbacks, session detection, settlement, and reviews

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
- frontend sends a TON Connect transaction to deploy/fund the escrow contract
- Telegram Wallet can approve the testnet transaction and return control to the Mini App
- after wallet return, the customer sees a checking-payment modal while backend verifies escrow funding
- after backend verification succeeds, the customer sees a payment-confirmed modal and can close the Telegram Mini App
- backend payment confirmation endpoint checks escrow contract state through TON Worker
- successful escrow funding moves payment and booking to `funded`
- backend sends Telegram Bot messages after successful funding verification
- expert confirmation deadline calculation implemented
- automatic booking timeout worker checks expired confirmations
- expired bookings automatically trigger `expert_decline` through TON Worker
- automatic refund flow updates booking/payment state
- Telegram Bot notifies both customer and expert when confirmation expires
- late Confirm/Decline button presses are safely rejected
- extensive timeout and payment lifecycle logging added
- frontend shows a wallet-return checking modal while backend verifies the escrow
- frontend shows a final payment-confirmed modal after backend verification succeeds

Currently still not finished:

- expert Confirm/Decline callback handling still needs final end-to-end hardening
- `expert_confirm` / `expert_decline` TON Worker actions still need final full-flow verification against funded escrow contracts
- full session detection and later settlement actions are still pending
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

Current behavior:

* wallet approval and return to the Mini App works in the testnet flow
* after wallet return, the frontend immediately shows a checking-payment modal
* the frontend calls `POST /api/bookings/{booking_id}/confirm-payment`
* the backend checks the escrow contract state through TON Worker
* if the contract is active and funded, backend moves `payment.status` and `booking.status` to `funded`
* after successful funding verification, backend sends Telegram Bot notifications
* the customer sees a final payment-confirmed modal

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


### Expert confirmation deadline rules

After the customer funds the escrow contract, the expert must confirm or decline the booking before the expert confirmation deadline.

Public availability rules:

```text
1. Public availability never shows slots closer than 4 hours.

2. Booking creation also rejects slots closer than 4 hours.

3. If the slot starts 24h+ from now:
   expert must answer within 24h.

4. If the slot starts between 4h and 24h from now:
   expert must answer within 4h.

5. Expert confirmation deadline is always capped at slot_start - 30 minutes.
```

Timeout behavior:

```text
If expert does not answer before deadline:
    backend calls TON Worker action expert_decline
    booking.status = refunded
    payment.status = refunded
    booking.expert_rejected_at = now
    rejected_reason = expert_response_timeout

Customer Telegram message:
    “The expert did not answer in time. Your escrow refund has been triggered.”

Expert Telegram message:
    “You did not answer in time. The escrow refund has been triggered.”
```

Late button behavior:

```text
If expert clicks Confirm or Decline after timeout:
    backend does not process the original action again
    Telegram callback alert says:
    “Too late. This request has already been refunded.”
```

Session outcome deadline:

```text
session_outcome_deadline_unix = slot_end + 10 minutes
```

This is separate from expert confirmation. It is used later for final session outcome settlement after the scheduled call window.

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
6. Rust backend validates selected slot against current availability
7. Rust backend creates `bookings` row with status `requested`
8. frontend begins payment
9. Rust backend verifies booking ownership by Telegram id
10. Rust backend updates booking to `awaiting_payment`
11. Rust backend creates or updates `payments` row with status `awaiting_payment`
12. if booking currency is `USD`, backend fetches TON/USD rate and converts quote to TON
13. backend builds TON Worker prepare payload
14. backend calls `POST /contracts/prepare-booking`
15. TON Worker prepares unique deterministic contract address and StateInit
16. backend stores returned contract address / transaction reference
17. backend returns payment data to frontend
18. frontend calls TON Connect `sendTransaction()`
19. Telegram Wallet approves the testnet transaction
20. Telegram Wallet returns control to the Mini App
21. frontend immediately closes the booking-confirm modal and opens the payment-checking modal
22. frontend calls `POST /api/bookings/{booking_id}/confirm-payment`
23. backend asks TON Worker for escrow contract state
24. backend verifies that the contract is active and funded
25. backend updates `payments.status = funded`
26. backend updates `bookings.status = funded`
27. backend sends Telegram Bot notification messages
28. frontend closes the checking modal and opens the payment-confirmed modal
29. customer can close the Telegram Mini App

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
When this succeeds, the customer wallet deploys and funds the prepared escrow contract.

    Important frontend rule:

    Do not treat “wallet opened” as payment success.
    Only treat “wallet returned + backend verified funded contract” as payment confirmation.

    Current modal flow:

    Confirm prepayment modal
↓
Telegram Wallet opens
↓
Wallet returns control
↓
Checking payment modal
↓
Backend /confirm-payment verifies escrow state
↓
Payment confirmed modal

Current backend verification source:

    TON Worker /contracts/{contract_address}/state

Example verified state:
```
{
    "account_state": "active",
    "balance_nano_ton": "188000938",
    "contract_state": 1,
    "contract_amount_nano_ton": "38167939",
    "is_funded": true
}
```
Expected successful status transition:

    booking.status: requested -> awaiting_payment -> funded
payment.status: awaiting_payment -> funded

---


## Frontend payment modal flow

The public expert page uses three payment-related modals.

### 1. Confirm prepayment modal

Existing modal:

```html
<div id="booking-confirm-modal" class="eh-modal-overlay hidden">
...
</div>
```
Purpose:

    shows selected expert, duration, slot, and quoted prepayment
starts the booking/payment flow when customer confirms

---

## TON Worker payload contract

Current Rust → TON Worker prepare payload should use the worker’s expected request shape.

Current target shape depends on the running TON Worker version. The Rust backend and worker DTOs must match exactly. One worker version expects:

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
The current Rust integration may use `amount` + `currency` if the running TON Worker schema expects that version. Do not change this casually: if wallet payment reaches Telegram Wallet and returns successfully, the Rust and worker payloads are already compatible for the active environment.

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

2. Checking payment modal

Used after Telegram Wallet returns control to the Mini App.

Required IDs:

<div id="payment-checking-modal" class="eh-modal-overlay hidden">
    <div class="eh-modal" role="dialog" aria-modal="true" aria-labelledby="payment-checking-title">
        <h3 id="payment-checking-title">Checking payment</h3>

        <p class="section-note" style="margin-top: 8px;">
            Telegram Wallet returned control to Expert Hub.
        </p>

        <p id="payment-checking-detail" class="section-note" style="margin-top: 8px;">
            Checking escrow contract funding status…
        </p>

        <div class="btn-grid" style="margin-top: 14px;">
            <button id="payment-checking-close" class="btn btn-secondary hidden" type="button">
                Close
            </button>
        </div>
    </div>
</div>

Purpose:

confirms that wallet control returned
calls backend /confirm-payment
shows retry/checking progress
only shows Close button if backend confirmation fails after retries
3. Payment confirmed modal

Used only after backend confirms funded escrow state.

Required IDs:
```
<div id="payment-confirmed-modal" class="eh-modal-overlay hidden">
    <div class="eh-modal" role="dialog" aria-modal="true" aria-labelledby="payment-confirmed-title">
        <h3 id="payment-confirmed-title">Payment confirmed</h3>

        <p class="section-note" style="margin-top: 8px;">
            The escrow contract is funded.
        </p>

        <p class="section-note" style="margin-top: 8px;">
            We notified the expert. Waiting for expert confirmation.
        </p>

        <div class="btn-grid">
            <button id="payment-confirmed-ok" class="btn btn-primary" type="button">OK</button>
        </div>
    </div>
</div>
```
Purpose:

```
tells customer that escrow funding was verified
lets customer close the Mini App
```

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

```
expert_decline    -> refund customer
expert_no_show    -> refund customer
session_connected -> pay expert
customer_no_show  -> pay expert
```

The TON Worker does not decide these outcomes. The backend decides the outcome and instructs the TON Worker.

---
## Telegram Bot confirmation flow

Normal product communication uses the official Telegram Bot API.

Telethon / research-service logic is not used for customer or expert notification messages. It is reserved only for future session/call detection if needed.

After the customer wallet transaction returns, the backend must verify the escrow contract before contacting the expert.

Verification checks:

```
contract is active
getState == 1
getBookingId == booking.id
contract balance / amount check passes
```
After successful verification:

payment.status = funded
booking.status = funded

Then the backend sends a Telegram bot message to the expert:

New booking request:

Customer: @username / Name
Date: 1 May 2026
Time: 14:00–14:30
Duration: 30 minutes
Amount: 0.187 TON

The customer has funded the escrow contract for this slot.

Do you confirm this booking?

Inline buttons:

✅ Confirm booking
❌ Decline booking

Callback data:

booking_confirm:{booking_id}:{payment_id}
booking_decline:{booking_id}:{payment_id}

Confirm behavior:

backend verifies callback sender is the expert
backend verifies booking.status = funded
backend verifies payment.status = funded
backend calls TON Worker action expert_confirm
booking.status = waiting_for_session
booking.expert_confirmed_at = now
customer is notified: expert has confirmed

Decline behavior:

backend verifies callback sender is the expert
backend verifies booking.status = funded
backend verifies payment.status = funded
backend calls TON Worker action expert_decline
booking.status = refunded
payment.status = refunded
booking.expert_rejected_at = now
booking.rejected_reason = expert_declined
customer is notified: expert declined, escrow refund triggered

## Expert confirmation deadline rules

After the customer funds the escrow contract, the expert must confirm or decline the booking before the expert confirmation deadline.

Public availability rules:

```text
1. Public availability never shows slots closer than 4 hours.

2. Booking creation also rejects slots closer than 4 hours.

3. If the slot starts 24h+ from now:
   expert must answer within 24h.

4. If the slot starts between 4h and 24h from now:
   expert must answer within 4h.

5. Expert confirmation deadline is always capped at slot_start - 30 minutes.
```

Timeout behavior:

```text
If expert does not answer before deadline:
    backend calls TON Worker action expert_decline
    booking.status = refunded
    payment.status = refunded
    booking.expert_rejected_at = now
    rejected_reason = expert_response_timeout

Customer Telegram message:
    "The expert did not answer in time. Your escrow refund has been triggered."

Expert Telegram message:
    "You did not answer in time. The escrow refund has been triggered."
```

Late button behavior:

```text
If expert clicks Confirm or Decline after timeout:
    backend does not process the original action again

Telegram callback alert:
    "Too late. This request has already been refunded."
```

Session outcome deadline:

```text
session_outcome_deadline_unix = slot_end + 10 minutes
```

This deadline is independent from expert confirmation.
It will later be used for session settlement after the scheduled consultation window.


Status path:

requested
↓
awaiting_payment
↓
funded
↓ expert confirms
waiting_for_session

Decline path:

requested
↓
awaiting_payment
↓
funded
↓ expert declines
refunded

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
16. frontend sends TON Connect escrow deployment/funding transaction
17. Telegram Wallet approves the testnet transaction and returns control to the Mini App
18. frontend shows checking-payment modal after wallet return
19. frontend calls backend payment confirmation endpoint
20. backend checks contract state through TON Worker
21. backend confirms active/funded escrow state
22. backend moves booking/payment to `funded`
23. backend sends Telegram Bot messages
24. frontend shows payment-confirmed modal
25. customer can close the Telegram Mini App
26. expert receives Telegram Confirm / Decline buttons
27. backend calculates expert confirmation deadline
28. background worker monitors expired confirmations
29. expired bookings automatically trigger expert_decline
30. booking/payment become refunded
31. both users receive timeout notifications
32. late Telegram callbacks are rejected safely

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
* `expert-public.js` handles slot selection, booking request, payment preparation, TON Connect transaction, wallet-return checking modal, backend payment confirmation call, payment-confirmed modal, and Telegram Mini App close action
* `app-config.js` controls dev/prod bot and TON network split


## 6. the expert-public.js refactoring plan

### Planned `expert-public.js` refactor

`public/js/expert-public.js` has grown too large and now mixes several responsibilities:

* DOM binding
* debug panel
* public expert rendering
* reviews rendering
* availability loading
* slot selection
* duration picker
* owner edit-button visibility
* TON Connect initialization
* booking request creation
* payment preparation
* wallet transaction sending
* payment confirmation polling
* modal state management

The file should be split into smaller modules.

Suggested target structure:

```
public/js/expert-public/
  index.js
  state.js
  dom.js
  debug.js
  api.js
  render-expert.js
  render-availability.js
  render-reviews.js
  booking-ui.js
  booking-api.js
  ton-connect.js
  payment-flow.js
  modals.js
```
Suggested responsibilities:
```
index.js
  boot sequence
  init Telegram WebApp
  bind events
  load initial page

state.js
  selectedSlot
  currentExpert
  currentWalletAddress
  currentSlug
  currentDurationMinutes
  currentOffsetDays
  request controller / request id

dom.js
  bindDom()
  exports els

debug.js
  ensureDebugPanel()
  debugBooking()
  setDebugStatus()
  installGlobalErrorLogging()

api.js
  loadPublicPage()
  loadAvailabilityOnly()

render-expert.js
  renderExpert()
  updateEditProfileButton()

render-availability.js
  renderSlots()
  renderDurationPicker()
  syncSelectedSlotWithPayload()

render-reviews.js
  renderReviews()

booking-ui.js
  deriveQuote()
  updateBookingUi()
  openBookingConfirmModal()

booking-api.js
  createBookingRequest()
  beginBookingPayment()
  confirmBookingPayment()
  confirmBookingPaymentWithRetry()

ton-connect.js
  initBookingTonConnect()
  sendTonBookingPayment()
  safeWalletSnapshot()
  describeTonChain()
  forceDisconnectWallet()

payment-flow.js
  orchestrates:
    create booking
    begin payment
    send TON transaction
    handle wallet return
    open checking modal
    call backend confirmation
    open confirmed modal

modals.js
  ensurePaymentModals()
  openModal()
  closeModal()
  setPaymentCheckingDetail()
  show/hide checking close button
```

Important refactor rule:

Do not change the working payment sequence while splitting the file.
First split without behavior changes. Then improve.

Current working payment sequence that must be preserved:

create booking
→ begin payment
→ send TON transaction
→ wallet returns control
→ show payment-checking modal immediately
→ call backend /confirm-payment
→ backend checks contract state through TON Worker
→ show payment-confirmed modal


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

## What is still ahead

Next priorities:

1. Complete the expert confirmation flow (booking status transition, customer notification, calendar integration if required).
2. Verify expert callback sender identity.
3. Verify `expert_confirm` action end-to-end against a funded escrow contract.
4. Verify `expert_decline` action end-to-end against a funded escrow contract.
5. Verify automatic timeout refund flow against the TON testnet.
6. Add Telegram watcher / research service.
7. Store watcher events in `telegram_call_events`.
8. Apply final session outcomes.
9. Wire `session_connected`, `customer_no_show`, and `expert_no_show` contract actions.
10. Add review flow.
11. Refactor `expert-public.js` into smaller modules without changing working behavior.
12. Add marketplace discovery later.

---

## Recent work summary

Recent booking/payment work completed:

* connected frontend booking flow to TON Connect transaction sending
* changed payment target from expert wallet to deterministic escrow contract address
* included `stateInit` in TON Connect transaction so the wallet can deploy/fund the escrow contract
* fixed TON Worker RPC provider key issue for testnet contract state checks
* confirmed funded testnet escrow contract state through TON Worker
* added backend `/confirm-payment` flow that verifies contract state
* confirmed manual payment verification updates booking/payment to `funded`
* fixed active Telegram bot token selection for dev bot environment
* verified Telegram Bot API sending through `TELEGRAM_DEV_BOT_TOKEN`
* added backend notification behavior after successful funding verification
* changed frontend flow from “wallet returned = success” to “wallet returned = show checking modal and ask backend”
* removed frontend recovery-style payment confirmation logic
* added payment-checking modal and payment-confirmed modal
* confirmed the direct wallet-return → backend-check → confirmed-modal flow works

Recent booking confirmation work completed:

* implemented expert confirmation deadline calculation
* implemented automatic booking timeout worker
* implemented automatic `expert_decline` execution through TON Worker
* implemented automatic refund state transition
* added `expert_response_timeout` rejection reason
* implemented timeout notifications for both customer and expert
* implemented protection against late Confirm/Decline callbacks
* added extensive logging for booking deadlines
* added extensive logging for timeout processing
* added extensive logging for TON payment and contract actions

Current limitation:

```text
Full end-to-end verification against the TON testnet is temporarily blocked because the public TON testnet has stopped processing ordinary user transactions.

The booking timeout, refund and confirmation logic has been implemented, but final end-to-end validation against a live escrow contract is pending until the public testnet resumes normal operation.
```

Important debugging lesson:

```
Frontend should not decide final payment status.
Wallet return only means the wallet handed control back.
Backend must verify escrow contract state before booking/payment becomes funded.
```

## Related service docs

Detailed TON Worker implementation notes belong in the separate worker repository README:

```text
ton-worker-experthub/README.md
```


---