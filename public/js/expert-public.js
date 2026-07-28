import {
    initTelegramWebApp,
    resolveTelegramUser
} from '/js/shared/telegram-auth.js';

import {
    TON_APP_NETWORK,
    BOT
} from '/js/shared/app-config.js';

// Hardcoded for current payment testing.
// TonConnect uses numeric chain IDs:
// -3   = testnet
// -239 = mainnet
const ACTIVE_TON_NETWORK_LABEL = 'TESTNET';
const ACTIVE_TON_CHAIN = '-3';

let els = {};

let selectedSlot = null;
let currentExpert = null;
let currentTonConnectUi = null;
let currentWalletAddress = '';

let currentOffsetDays = 0;
let currentSlug = null;
let currentDurationMinutes = null;
let expertAllowedDurations = [];

let currentPageRequestController = null;
let currentPageRequestId = 0;

let currentPayment = null;

function bindDom() {
    els = {
        debugStatus: document.getElementById('debug-status'),

        expertName: document.getElementById('expert-name'),
        expertHeadline: document.getElementById('expert-headline'),
        expertAvatar: document.getElementById('expert-avatar'),
        expertAvatarPlaceholder: document.getElementById('expert-avatar-placeholder'),
        expertUsername: document.getElementById('expert-username'),
        expertTimezone: document.getElementById('expert-timezone'),
        expertRating: document.getElementById('expert-rating'),
        expertRate: document.getElementById('expert-rate'),
        expertDurations: document.getElementById('expert-durations'),
        expertBio: document.getElementById('expert-bio'),

        slotsRangeLabel: document.getElementById('slots-range-label'),
        slotsList: document.getElementById('slots-list'),
        slotsEmpty: document.getElementById('slots-empty'),

        reviewsList: document.getElementById('reviews-list'),
        reviewsEmpty: document.getElementById('reviews-empty'),

        prevPeriodBtn: document.getElementById('prev-period-btn'),
        nextPeriodBtn: document.getElementById('next-period-btn'),

        durationPickerWrap: document.getElementById('duration-picker-wrap'),
        durationPicker: document.getElementById('duration-picker'),

        editProfileBtn: document.getElementById('edit-profile-btn'),

        bookBtn: document.getElementById('book-btn'),
        selectedSlotSummary: document.getElementById('selected-slot-summary'),

        telegramRequiredModal: document.getElementById('telegram-required-modal'),
        walletRequiredModal: document.getElementById('wallet-required-modal'),
        bookingConfirmModal: document.getElementById('booking-confirm-modal'),

        telegramRequiredClose: document.getElementById('telegram-required-close'),
        walletRequiredClose: document.getElementById('wallet-required-close'),
        bookingConfirmClose: document.getElementById('booking-confirm-close'),
        bookingConfirmSubmit: document.getElementById('booking-confirm-submit'),
        bookingConfirmText: document.getElementById('booking-confirm-text'),

        paymentCheckingModal: document.getElementById('payment-checking-modal'),
        paymentCheckingDetail: document.getElementById('payment-checking-detail'),
        paymentCheckingClose: document.getElementById('payment-checking-close'),

        paymentConfirmedModal: document.getElementById('payment-confirmed-modal'),
        paymentConfirmedOk: document.getElementById('payment-confirmed-ok')
    };
}

function ensurePaymentModals() {
    let checkingModal = document.getElementById('payment-checking-modal');

    if (!checkingModal) {
        checkingModal = document.createElement('div');
        checkingModal.id = 'payment-checking-modal';
        checkingModal.className = 'eh-modal-overlay hidden';

        checkingModal.innerHTML = `
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
        `;

        document.body.appendChild(checkingModal);
    }

    let confirmedModal = document.getElementById('payment-confirmed-modal');

    if (!confirmedModal) {
        confirmedModal = document.createElement('div');
        confirmedModal.id = 'payment-confirmed-modal';
        confirmedModal.className = 'eh-modal-overlay hidden';

        confirmedModal.innerHTML = `
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
        `;

        document.body.appendChild(confirmedModal);
    }

    els.paymentCheckingModal = checkingModal;
    els.paymentCheckingDetail = document.getElementById('payment-checking-detail');
    els.paymentCheckingClose = document.getElementById('payment-checking-close');

    els.paymentConfirmedModal = confirmedModal;
    els.paymentConfirmedOk = document.getElementById('payment-confirmed-ok');
}

function setPaymentCheckingDetail(text) {
    if (els.paymentCheckingDetail) {
        els.paymentCheckingDetail.textContent = text;
    }
}

function showPaymentCheckingCloseButton() {
    els.paymentCheckingClose?.classList.remove('hidden');
}

function hidePaymentCheckingCloseButton() {
    els.paymentCheckingClose?.classList.add('hidden');
}

function ensureDebugPanel() {
    let panel = document.getElementById('booking-debug-panel');

    if (!panel) {
        panel = document.createElement('pre');
        panel.id = 'booking-debug-panel';
        document.body.appendChild(panel);
    }

    return panel;
}

function safeJson(value) {
    try {
        return JSON.stringify(value, null, 2);
    } catch (error) {
        return String(value);
    }
}

function debugBooking(label, data = null) {
    const panel = ensureDebugPanel();

    const line = `[${new Date().toISOString()}] ${label}` +
        (data ? `\n${safeJson(data)}` : '');

    panel.textContent = `${line}\n\n${panel.textContent || ''}`;
}

function setDebugStatus(text) {
    if (els.debugStatus) {
        els.debugStatus.textContent = text || '';
    }

    if (text) {
        debugBooking(`STATUS: ${text}`);
    }
}

function installGlobalErrorLogging() {
    window.addEventListener('error', (event) => {
        debugBooking('window.error', {
            message: event.message,
            filename: event.filename,
            lineno: event.lineno,
            colno: event.colno,
            error: event.error?.stack || String(event.error || '')
        });

        setDebugStatus(event.message || 'JavaScript error.');
    });

    window.addEventListener('unhandledrejection', (event) => {
        debugBooking('window.unhandledrejection', {
            reason: event.reason?.message || String(event.reason),
            stack: event.reason?.stack || null
        });

        setDebugStatus(event.reason?.message || 'Unhandled promise rejection.');
    });
}

function cancelCurrentPageRequest() {
    if (currentPageRequestController) {
        currentPageRequestController.abort();
        currentPageRequestController = null;
    }
}

function updatePeriodButtonsState(isLoading = false) {
    if (!els.prevPeriodBtn || !els.nextPeriodBtn) return;

    els.prevPeriodBtn.disabled = isLoading || currentOffsetDays <= 0;
    els.nextPeriodBtn.disabled = isLoading;
}

function syncSelectedSlotWithPayload(payload) {
    if (!selectedSlot || !Array.isArray(payload?.days)) {
        return;
    }

    const selectedKey = `${selectedSlot.start_utc}|${selectedSlot.duration_minutes}`;
    let stillExists = false;

    for (const day of payload.days) {
        for (const slot of day.slots || []) {
            const slotKey = `${slot.start_utc}|${slot.duration_minutes}`;

            if (slotKey === selectedKey) {
                stillExists = true;
                break;
            }
        }

        if (stillExists) {
            break;
        }
    }

    if (!stillExists) {
        selectedSlot = null;
    }
}

function getSlugFromPath() {
    const parts = window.location.pathname.split('/').filter(Boolean);

    if (parts.length >= 2 && parts[0] === 'e') {
        return parts[1];
    }

    return '';
}

function formatMoney(amount, currency) {
    return `${amount} ${currency}`;
}

function formatDurations(durations) {
    if (!Array.isArray(durations) || !durations.length) {
        return '—';
    }

    return durations.map((value) => `${value} min`).join(', ');
}

function formatDateRangeLabel(startDate, endDate) {
    return `${startDate} → ${endDate}`;
}

function normalizeDurations(durations) {
    if (!Array.isArray(durations)) {
        return [];
    }

    return durations
        .map((value) => Number(value))
        .filter((value) => Number.isInteger(value) && value > 0)
        .sort((a, b) => a - b);
}

function renderExpert(data) {
    els.expertName.textContent = data.display_name || 'Expert';
    els.expertHeadline.textContent = data.telegram_bio || 'Expert profile';
    els.expertUsername.textContent = data.username ? `@${data.username}` : 'No public username';
    els.expertTimezone.textContent = `Timezone: ${data.timezone}`;
    els.expertRating.textContent = data.reviews_count > 0
        ? `${data.expert_rating}/5 · ${data.reviews_count} review(s)`
        : 'No reviews yet';

    els.expertRate.textContent = formatMoney(data.hourly_rate, data.currency);
    els.expertDurations.textContent = formatDurations(data.allowed_session_durations);
    els.expertBio.textContent = data.telegram_bio || 'No description yet.';

    els.expertAvatarPlaceholder.textContent =
        (data.display_name || 'E').trim().charAt(0).toUpperCase() || 'E';

    if (data.photo_url) {
        els.expertAvatar.src = data.photo_url;
        els.expertAvatar.classList.remove('hidden');
        els.expertAvatarPlaceholder.classList.add('hidden');
    } else {
        els.expertAvatar.classList.add('hidden');
        els.expertAvatarPlaceholder.classList.remove('hidden');
    }

    updateEditProfileButton(data);
}

function renderSlots(payload) {
    els.slotsRangeLabel.textContent = formatDateRangeLabel(payload.period_start, payload.period_end);
    els.slotsList.innerHTML = '';

    if (!Array.isArray(payload.days) || !payload.days.length) {
        els.slotsEmpty.classList.remove('hidden');
        selectedSlot = null;
        updateBookingUi();
        return;
    }

    const visibleDays = payload.days.filter((day) => Array.isArray(day.slots) && day.slots.length > 0);

    if (!visibleDays.length) {
        els.slotsEmpty.classList.remove('hidden');
        selectedSlot = null;
        updateBookingUi();
        return;
    }

    els.slotsEmpty.classList.add('hidden');

    visibleDays.forEach((day) => {
        const card = document.createElement('div');
        card.className = 'slot-day-card';

        const title = document.createElement('div');
        title.className = 'slot-day-title';
        title.textContent = day.label;

        const chipList = document.createElement('div');
        chipList.className = 'slot-chip-list';

        day.slots.forEach((slot) => {
            const chip = document.createElement('button');
            chip.type = 'button';
            chip.className = 'slot-chip';
            chip.textContent = `${slot.start_local} · ${slot.duration_minutes} min`;

            const slotKey = `${slot.start_utc}|${slot.duration_minutes}`;
            const selectedKey = selectedSlot
                ? `${selectedSlot.start_utc}|${selectedSlot.duration_minutes}`
                : null;

            if (selectedKey === slotKey) {
                chip.classList.add('is-selected');
            }

            chip.addEventListener('click', () => {
                selectedSlot = {
                    ...slot,
                    day_label: day.label
                };

                debugBooking('slot selected', selectedSlot);

                renderSlots(payload);
                updateBookingUi();
            });

            chipList.appendChild(chip);
        });

        card.appendChild(title);
        card.appendChild(chipList);
        els.slotsList.appendChild(card);
    });
}

function renderReviews(reviews) {
    els.reviewsList.innerHTML = '';

    if (!Array.isArray(reviews) || !reviews.length) {
        els.reviewsEmpty.classList.remove('hidden');
        return;
    }

    els.reviewsEmpty.classList.add('hidden');

    reviews.forEach((review) => {
        const card = document.createElement('div');
        card.className = 'review-card';

        const head = document.createElement('div');
        head.className = 'review-head';

        const author = document.createElement('div');
        author.className = 'review-author';
        author.textContent = review.author_telegram_name || 'Anonymous';

        const rating = document.createElement('div');
        rating.className = 'review-rating';
        rating.textContent = `${review.review_rating}/5`;

        const text = document.createElement('div');
        text.className = 'review-text';
        text.textContent = review.review_text || 'No text review.';

        head.appendChild(author);
        head.appendChild(rating);

        card.appendChild(head);
        card.appendChild(text);

        els.reviewsList.appendChild(card);
    });
}

async function loadPublicPage() {
    if (!currentSlug) {
        setDebugStatus('Missing slug in URL.');
        return;
    }

    cancelCurrentPageRequest();

    const requestId = ++currentPageRequestId;
    const controller = new AbortController();
    currentPageRequestController = controller;

    setDebugStatus('Loading public page…');
    updatePeriodButtonsState(true);

    try {
        const response = await fetch(
            `/api/experts/${encodeURIComponent(currentSlug)}/public?offset_days=${currentOffsetDays}`,
            { signal: controller.signal }
        );

        const payload = await response.json().catch(() => ({}));

        debugBooking('loadPublicPage: response', {
            status: response.status,
            ok: response.ok,
            payload
        });

        if (!response.ok) {
            throw new Error(payload.message || 'Failed to load public page.');
        }

        if (requestId !== currentPageRequestId) {
            return;
        }

        currentExpert = payload.expert;

        renderExpert(payload.expert);
        renderReviews(payload.reviews || []);

        expertAllowedDurations = normalizeDurations(payload.expert?.allowed_session_durations);

        if (
            !currentDurationMinutes ||
            !expertAllowedDurations.includes(currentDurationMinutes)
        ) {
            currentDurationMinutes = expertAllowedDurations.length
                ? expertAllowedDurations[0]
                : null;
        }

        renderDurationPicker(expertAllowedDurations);

        if (currentDurationMinutes) {
            syncSelectedSlotWithPayload(payload.availability);
            renderSlots(payload.availability);
            updateBookingUi();
        } else {
            els.slotsList.innerHTML = '';
            els.slotsRangeLabel.textContent = '—';
            els.slotsEmpty.classList.remove('hidden');
            selectedSlot = null;
            updateBookingUi();
        }

        setDebugStatus('');
    } catch (error) {
        if (error.name === 'AbortError') {
            return;
        }

        if (requestId !== currentPageRequestId) {
            return;
        }

        debugBooking('loadPublicPage: ERROR', {
            message: error?.message,
            stack: error?.stack
        });

        setDebugStatus(error.message || 'Failed to load public page.');
    } finally {
        if (requestId === currentPageRequestId) {
            currentPageRequestController = null;
            updatePeriodButtonsState(false);
        }
    }
}

function renderDurationPicker(durations) {
    els.durationPicker.innerHTML = '';

    if (!Array.isArray(durations) || !durations.length) {
        els.durationPickerWrap.classList.add('hidden');
        return;
    }

    els.durationPickerWrap.classList.remove('hidden');

    durations.forEach((duration) => {
        const chip = document.createElement('button');
        chip.type = 'button';
        chip.className = 'duration-chip';
        chip.textContent = `${duration} min`;

        if (duration === currentDurationMinutes) {
            chip.classList.add('active');
        }

        chip.addEventListener('click', async (event) => {
            event.preventDefault();
            event.stopPropagation();

            if (duration === currentDurationMinutes) {
                return;
            }

            currentDurationMinutes = duration;
            renderDurationPicker(expertAllowedDurations);
            await loadAvailabilityOnly();
        });

        els.durationPicker.appendChild(chip);
    });
}

async function loadAvailabilityOnly() {
    if (!currentSlug || !currentDurationMinutes) {
        return;
    }

    cancelCurrentPageRequest();

    const requestId = ++currentPageRequestId;
    const controller = new AbortController();

    currentPageRequestController = controller;
    updatePeriodButtonsState(true);

    try {
        setDebugStatus('Loading availability…');

        const response = await fetch(
            `/api/experts/${encodeURIComponent(currentSlug)}/public?offset_days=${currentOffsetDays}&duration_minutes=${currentDurationMinutes}`,
            { signal: controller.signal }
        );

        const payload = await response.json().catch(() => ({}));

        debugBooking('loadAvailabilityOnly: response', {
            status: response.status,
            ok: response.ok,
            payload
        });

        if (!response.ok) {
            throw new Error(payload.message || 'Failed to load availability.');
        }

        if (requestId !== currentPageRequestId) {
            return;
        }

        syncSelectedSlotWithPayload(payload.availability);
        renderSlots(payload.availability);
        updateBookingUi();
        setDebugStatus('');
    } catch (error) {
        if (error.name === 'AbortError') {
            return;
        }

        if (requestId !== currentPageRequestId) {
            return;
        }

        debugBooking('loadAvailabilityOnly: ERROR', {
            message: error?.message,
            stack: error?.stack
        });

        setDebugStatus(error.message || 'Failed to load availability.');
    } finally {
        if (requestId === currentPageRequestId) {
            currentPageRequestController = null;
            updatePeriodButtonsState(false);
        }
    }
}

function updateEditProfileButton(data) {
    if (!els.editProfileBtn) {
        return;
    }

    els.editProfileBtn.href = `/e/${encodeURIComponent(data.public_slug)}/edit`;

    const telegramUser = resolveTelegramUser();

    const isOwner =
        telegramUser &&
        Number(telegramUser.id) > 0 &&
        Number(data.telegram_id) === Number(telegramUser.id);

    if (isOwner) {
        els.editProfileBtn.classList.remove('hidden');
    } else {
        els.editProfileBtn.classList.add('hidden');
    }
}

function closeModal(el) {
    el?.classList.add('hidden');
}

function openModal(el) {
    el?.classList.remove('hidden');
}

function closeTelegramMiniAppOrGoHome() {
    const tg = window.Telegram?.WebApp;

    debugBooking('closeTelegramMiniAppOrGoHome: called', {
        hasTelegramWebApp: !!tg,
        hasCloseFunction: typeof tg?.close === 'function'
    });

    if (tg && typeof tg.close === 'function') {
        tg.close();
        return;
    }

    window.location.href = '/';
}

function formatSlotLabel(slot) {
    return `${slot.day_label || ''} · ${slot.start_local || slot.start_utc || ''}`;
}

function deriveQuote(expert, durationMinutes) {
    const rate = Number(expert?.hourly_rate || 0);
    const quote = (rate * Number(durationMinutes || 0)) / 60;
    return Number.isFinite(quote) ? quote.toFixed(2) : '0.00';
}

function updateBookingUi() {
    if (!els.bookBtn) return;

    const enabled = !!selectedSlot && !!currentDurationMinutes && !!currentExpert;

    els.bookBtn.disabled = !enabled;

    if (!enabled) {
        els.selectedSlotSummary?.classList.add('hidden');

        if (els.selectedSlotSummary) {
            els.selectedSlotSummary.textContent = '';
        }

        return;
    }

    const quote = deriveQuote(currentExpert, currentDurationMinutes);

    els.selectedSlotSummary?.classList.remove('hidden');
    els.selectedSlotSummary.textContent =
        `Selected: ${formatSlotLabel(selectedSlot)} · ${currentDurationMinutes} min · ${quote} ${currentExpert.currency}`;
}

function safeWalletSnapshot(wallet) {
    if (!wallet) {
        return null;
    }

    return {
        device: wallet.device || null,
        provider: wallet.provider || null,
        account: wallet.account
            ? {
                address: wallet.account.address || null,
                chain: wallet.account.chain || null,
                publicKey: wallet.account.publicKey || null
            }
            : null
    };
}

function safeTonConnectUiSnapshot(tonConnectUi) {
    if (!tonConnectUi) {
        return null;
    }

    return {
        wallet: safeWalletSnapshot(tonConnectUi.wallet),
        hasConnectionRestoredPromise: tonConnectUi.connectionRestored instanceof Promise
    };
}

function describeTonChain(chain) {
    if (!chain) return 'unknown';

    switch (String(chain)) {
        case '-3':
            return 'testnet';
        case '-239':
            return 'mainnet';
        default:
            return String(chain);
    }
}

function initBookingTonConnect() {
    if (!window.TON_CONNECT_UI) {
        debugBooking('initBookingTonConnect: TON_CONNECT_UI is missing on window');
        return;
    }

    if (currentTonConnectUi) {
        debugBooking('initBookingTonConnect: already initialized', safeTonConnectUiSnapshot(currentTonConnectUi));
        return;
    }

    debugBooking('initBookingTonConnect: creating TonConnectUI', {
        manifestUrl: `${window.location.origin}/tonconnect-manifest.json`,
        importedNetwork: TON_APP_NETWORK,
        activeNetworkLabel: ACTIVE_TON_NETWORK_LABEL,
        activeChain: ACTIVE_TON_CHAIN,
        userAgent: navigator.userAgent,
        href: window.location.href,
        isTelegramMiniApp: !!window.Telegram?.WebApp,
        telegramVersion: window.Telegram?.WebApp?.version || null,
        telegramPlatform: window.Telegram?.WebApp?.platform || null,
        buttonRootExists: !!document.getElementById('booking-ton-connect')
    });

    currentTonConnectUi = new window.TON_CONNECT_UI.TonConnectUI({
        manifestUrl: `${window.location.origin}/tonconnect-manifest.json`,
        buttonRootId: 'booking-ton-connect',
        actionsConfiguration: {
            twaReturnUrl: `https://t.me/${BOT}?startapp=${currentSlug || 'expert_new'}`
        }
    });

    if (typeof currentTonConnectUi.setConnectRequestParameters === 'function') {
        currentTonConnectUi.setConnectRequestParameters({
            state: 'ready',
            value: {
                items: [
                    {
                        name: 'ton_addr',
                        network: ACTIVE_TON_CHAIN
                    }
                ]
            }
        });

        debugBooking('TonConnect connection request target applied', {
            requestedItem: {
                name: 'ton_addr',
                network: ACTIVE_TON_CHAIN
            }
        });
    } else {
        debugBooking('TonConnect setConnectRequestParameters is not available');
    }

    if (currentTonConnectUi.wallet?.account?.address) {
        currentWalletAddress = currentTonConnectUi.wallet.account.address;

        debugBooking('initBookingTonConnect: initial wallet found', {
            currentWalletAddress,
            chain: currentTonConnectUi.wallet?.account?.chain || null,
            chainLabel: describeTonChain(currentTonConnectUi.wallet?.account?.chain)
        });
    } else {
        debugBooking('initBookingTonConnect: no initial wallet connected');
    }

    if (currentTonConnectUi.connectionRestored instanceof Promise) {
        currentTonConnectUi.connectionRestored
            .then((restored) => {
                debugBooking('TonConnect connectionRestored resolved', {
                    restored,
                    snapshot: safeTonConnectUiSnapshot(currentTonConnectUi)
                });
            })
            .catch((error) => {
                debugBooking('TonConnect connectionRestored rejected', {
                    message: error?.message,
                    stack: error?.stack
                });
            });
    }

    currentTonConnectUi.onStatusChange(
        (wallet) => {
            currentWalletAddress = wallet?.account?.address || '';

            debugBooking('TonConnect onStatusChange', {
                wallet: safeWalletSnapshot(wallet),
                currentWalletAddress,
                chain: wallet?.account?.chain || null,
                chainLabel: describeTonChain(wallet?.account?.chain),
                expectedNetwork: ACTIVE_TON_NETWORK_LABEL,
                expectedChain: ACTIVE_TON_CHAIN
            });

            const actualChain = String(wallet?.account?.chain || '');
            const expectedChain = ACTIVE_TON_CHAIN;

            if (currentWalletAddress && actualChain !== expectedChain) {
                debugBooking('TonConnect wrong network on connect', {
                    currentWalletAddress,
                    actualChain,
                    actualChainLabel: describeTonChain(actualChain),
                    expectedChain,
                    expectedNetwork: ACTIVE_TON_NETWORK_LABEL
                });

                setDebugStatus(
                    `Wrong wallet network. Expected ${ACTIVE_TON_NETWORK_LABEL}, got ${describeTonChain(actualChain)}. Disconnecting wallet.`
                );

                currentWalletAddress = '';

                currentTonConnectUi.disconnect().catch((error) => {
                    debugBooking('TonConnect auto-disconnect ERROR', {
                        message: error?.message,
                        stack: error?.stack
                    });
                });

                return;
            }

            if (
                currentWalletAddress &&
                els.walletRequiredModal &&
                !els.walletRequiredModal.classList.contains('hidden')
            ) {
                closeModal(els.walletRequiredModal);
                openBookingConfirmModal();
            }
        },
        (error) => {
            debugBooking('TonConnect onStatusChange ERROR', {
                message: error?.message,
                stack: error?.stack
            });
        }
    );
}

function openBookingConfirmModal() {
    if (!selectedSlot || !currentExpert) {
        debugBooking('openBookingConfirmModal: blocked', {
            selectedSlot,
            currentExpert
        });
        return;
    }

    const quote = deriveQuote(currentExpert, currentDurationMinutes);

    els.bookingConfirmText.textContent =
        `You are booking ${currentExpert.display_name} for ${currentDurationMinutes} minutes at ${selectedSlot.day_label} ${selectedSlot.start_local}. Prepayment: ${quote} ${currentExpert.currency}.`;

    openModal(els.bookingConfirmModal);

    debugBooking('openBookingConfirmModal: opened', {
        selectedSlot,
        currentDurationMinutes,
        quote
    });
}

async function createBookingRequest() {
    const telegramUser = resolveTelegramUser();

    debugBooking('createBookingRequest: initial state', {
        telegramUser,
        currentSlug,
        selectedSlot,
        currentDurationMinutes,
        currentWalletAddress,
        currentExpertCurrency: currentExpert?.currency,
        currentExpertRate: currentExpert?.hourly_rate
    });

    if (!telegramUser?.id) {
        openModal(els.telegramRequiredModal);
        return null;
    }

    const payload = {
        expert_slug: currentSlug,
        slot_start: selectedSlot?.start_utc,
        duration_minutes: currentDurationMinutes,
        requested_by_telegram_id: Number(telegramUser.id),
        requested_by_username: telegramUser.username || null,
        requested_by_display_name: [telegramUser.first_name, telegramUser.last_name].filter(Boolean).join(' ') || telegramUser.first_name || 'Telegram user',
        requested_by_ton_wallet: currentWalletAddress || null
    };

    const response = await fetch('/api/bookings/request', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    });

    const data = await response.json().catch(() => ({}));

    debugBooking('createBookingRequest: response', {
        status: response.status,
        ok: response.ok,
        data
    });

    if (!response.ok) {
        throw new Error(data.error || data.message || 'Booking request failed.');
    }

    return data;
}

async function beginBookingPayment(bookingId) {
    const telegramUser = resolveTelegramUser();

    const payload = {
        telegram_id: Number(telegramUser?.id),
        ton_wallet_customer: currentWalletAddress
    };

    const response = await fetch(`/api/bookings/${encodeURIComponent(bookingId)}/begin-payment`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    });

    const data = await response.json().catch(() => ({}));

    debugBooking('beginBookingPayment: response', {
        bookingId,
        status: response.status,
        ok: response.ok,
        data
    });

    if (!response.ok) {
        throw new Error(data.error || data.message || 'Payment start failed.');
    }

    return data;
}

async function confirmBookingPayment(bookingId, txResult) {
    const payload = {
        boc: txResult?.boc || null,
        trace_id: txResult?.traceId || txResult?.boc || `wallet_returned_${Date.now()}`
    };

    debugBooking('confirmBookingPayment: backend check requested', {
        bookingId,
        hasBoc: !!payload.boc,
        bocLength: payload.boc ? String(payload.boc).length : 0,
        trace_id: payload.trace_id ? String(payload.trace_id).slice(0, 80) : null
    });

    const response = await fetch(`/api/bookings/${encodeURIComponent(bookingId)}/confirm-payment`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    });

    const data = await response.json().catch(() => ({}));

    debugBooking('confirmBookingPayment: backend response', {
        status: response.status,
        ok: response.ok,
        data
    });

    if (!response.ok) {
        throw new Error(data.error || data.message || 'Payment confirmation failed.');
    }

    return data;
}

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function confirmBookingPaymentWithRetry(bookingId, txResult) {
    let lastError = null;

    for (let attempt = 1; attempt <= 12; attempt += 1) {
        try {
            setDebugStatus(`Wallet returned. Checking payment… attempt ${attempt}/12`);
            setPaymentCheckingDetail(`Checking escrow contract funding status… attempt ${attempt}/12`);

            const result = await confirmBookingPayment(bookingId, txResult);

            const balance = BigInt(result.balance_nano_ton ?? '0');
            const expected = BigInt(currentPayment.total_deploy_value_nano_ton);
            const minimumAccepted = expected * 98n / 100n;

            if (
                result.verified === true &&
                result.account_state === 'active' &&
                balance >= minimumAccepted
            ) {
                debugBooking('SUCCESS RESPONSE FROM BACKEND', {
                        attempt,
                        response: result,
                        balance: balance.toString(),
                        expected: expected.toString(),
                        minimumAccepted: minimumAccepted.toString()
                });
                return result;
            }

            debugBooking('Payment not verified yet', {
                attempt,
                verified: result.verified,
                account_state: result.account_state,
                balance: balance.toString(),
                expected: expected.toString()
            });

            debugBooking('ABOUT TO THROW', result);

            throw new Error(
                `Contract state=${result.account_state}, verified=${result.verified}, balance=${balance}, expected=${expected}`
            );
        } catch (error) {
            lastError = error;

            debugBooking('confirmBookingPaymentWithRetry: attempt failed', {
                bookingId,
                attempt,
                message: error?.message
            });
        }

        if (attempt < 12) {
            await sleep(5000);
        }
    }

    throw lastError || new Error(
        'Payment could not be verified on the TON blockchain.'
    );
}

function normalizePaymentPayload(data) {
    if (!data) {
        return null;
    }

    const payload = data.payment_payload || data.ton_payment || data;

    const contractAddress =
        payload.contract_address ||
        payload.contractAddress ||
        payload.destination_address ||
        payload.destinationAddress ||
        data.contract_address ||
        data.contractAddress ||
        null;

    const amountNanoTon =
        payload.total_deploy_value_nano_ton ||
        payload.totalDeployValueNanoTon ||
        data.total_deploy_value_nano_ton ||
        data.totalDeployValueNanoTon ||
        payload.amount_nano_ton ||
        payload.amountNanoTon ||
        data.amount_nano_ton ||
        data.amountNanoTon ||
        null;

    const stateInitBoc =
        payload.state_init_boc ||
        payload.stateInitBoc ||
        payload.stateInit ||
        data.state_init_boc ||
        data.stateInitBoc ||
        data.stateInit ||
        null;

    return {
        booking_id:
            data.booking_id ||
            data.bookingId ||
            payload.booking_id ||
            payload.bookingId ||
            null,

        payment_id:
            data.payment_id ||
            data.paymentId ||
            payload.payment_id ||
            payload.paymentId ||
            payload.id ||
            null,

        contract_address: contractAddress,
        destination_address: contractAddress,

        amount_nano_ton: amountNanoTon,

        total_deploy_value_nano_ton:
            payload.total_deploy_value_nano_ton ??
            payload.totalDeployValueNanoTon ??
            data.total_deploy_value_nano_ton ??
            data.totalDeployValueNanoTon ??
            amountNanoTon,

        state_init_boc: stateInitBoc,

        raw: data
    };
}

async function sendTonBookingPayment(paymentPayload) {
    if (!currentTonConnectUi) {
        throw new Error('TON Connect is not initialized.');
    }

    if (!currentWalletAddress) {
        throw new Error('TON wallet is not connected.');
    }

    const destinationAddress =
        paymentPayload?.destination_address ||
        paymentPayload?.contract_address;

    const amountNanoTon = paymentPayload?.amount_nano_ton
        ? String(paymentPayload.amount_nano_ton)
        : '';

    if (!destinationAddress || !amountNanoTon) {
        throw new Error('Payment payload is missing destination address or amount.');
    }

    if (!/^\d+$/.test(amountNanoTon)) {
        throw new Error(`Payment amount must be an integer nanoTON string. Got: ${amountNanoTon}`);
    }

    if (!paymentPayload.state_init_boc) {
        throw new Error('Missing state_init_boc. Cannot deploy escrow contract.');
    }

    const actualChain = String(currentTonConnectUi?.wallet?.account?.chain || '');

    if (actualChain !== ACTIVE_TON_CHAIN) {
        throw new Error(
            `Wrong wallet network. Expected ${ACTIVE_TON_NETWORK_LABEL}, got ${describeTonChain(actualChain)} (${actualChain}).`
        );
    }

    const txPayload = {
        validUntil: Math.floor(Date.now() / 1000) + 300,
        network: ACTIVE_TON_CHAIN,
        messages: [
            {
                address: destinationAddress,
                amount: amountNanoTon,
                stateInit: paymentPayload.state_init_boc
            }
        ]
    };

    debugBooking('sendTonBookingPayment: sending transaction to wallet', {
        contractAddress: destinationAddress,
        amountNanoTon,
        amountTonApprox: Number(amountNanoTon) / 1_000_000_000,
        hasStateInit: true,
        stateInitLength: String(paymentPayload.state_init_boc).length
    });

    setDebugStatus('Opening Telegram Wallet…');

    const result = await currentTonConnectUi.sendTransaction(txPayload);

    debugBooking('sendTonBookingPayment: wallet returned control', {
        returnedAt: new Date().toISOString(),
        hasBoc: !!result?.boc,
        bocLength: result?.boc ? String(result.boc).length : 0,
        result
    });

    return result;
}

async function showReturnedFromWalletAndConfirmPayment(bookingId, txResult) {
    closeModal(els.bookingConfirmModal);

    hidePaymentCheckingCloseButton();
    openModal(els.paymentCheckingModal);

    setPaymentCheckingDetail(
        'Asking the backend to check the escrow contract funding status now…'
    );

    debugBooking('wallet returned: checking modal shown, calling backend now', {
        bookingId,
        returnedAt: new Date().toISOString(),
        hasBoc: !!txResult?.boc
    });

    try {
        const confirmed = await confirmBookingPaymentWithRetry(bookingId, txResult);

        debugBooking('ABOUT TO OPEN SUCCESS MODAL', confirmed);

        closeModal(els.paymentCheckingModal);

        selectedSlot = null;
        updateBookingUi();

        setDebugStatus('Payment confirmed. Waiting for expert confirmation.');

        debugBooking('OPENING PAYMENT CONFIRMED MODAL');

        openModal(els.paymentConfirmedModal);

        return confirmed;

    } catch (error) {

        debugBooking('payment verification FAILED', {
            bookingId,
            message: error?.message,
            stack: error?.stack
        });

        setDebugStatus(
            'Payment could not be verified on the blockchain.'
        );

        setPaymentCheckingDetail(
            'The wallet returned, but the escrow contract never became active.\n\n' +
            'Please verify the transaction in your wallet. ' +
            'If no transaction exists, the payment was not sent.'
        );

        showPaymentCheckingCloseButton();

        // IMPORTANT:
        // DO NOT close this modal.
        // DO NOT open the success modal.
        // DO NOT clear selectedSlot.
        throw error;
    }
}

function bindEvents() {
    els.bookBtn?.addEventListener('click', async (event) => {
        event.preventDefault();
        event.stopPropagation();

        try {
            const telegramUser = resolveTelegramUser();

            if (!telegramUser?.id) {
                openModal(els.telegramRequiredModal);
                return;
            }

            if (!selectedSlot || !currentExpert || !currentDurationMinutes) {
                throw new Error('Choose a slot first.');
            }

            if (!currentTonConnectUi) {
                openModal(els.walletRequiredModal);
                initBookingTonConnect();
                return;
            }

            if (!currentWalletAddress) {
                openModal(els.walletRequiredModal);
                return;
            }

            const actualChain = String(currentTonConnectUi?.wallet?.account?.chain || '');

            if (actualChain !== ACTIVE_TON_CHAIN) {
                throw new Error(
                    `Wrong wallet network. Expected ${ACTIVE_TON_NETWORK_LABEL}, got ${describeTonChain(actualChain)} (${actualChain}).`
                );
            }

            openBookingConfirmModal();
        } catch (error) {
            debugBooking('bookBtn click: ERROR', {
                message: error?.message,
                stack: error?.stack
            });

            setDebugStatus(error.message || 'Booking flow failed.');
        }
    });

    els.bookingConfirmSubmit?.addEventListener('click', async (event) => {
        event.preventDefault();
        event.stopPropagation();

        if (els.bookingConfirmSubmit.disabled) {
            return;
        }

        try {
            els.bookingConfirmSubmit.disabled = true;

            if (!selectedSlot || !currentExpert || !currentDurationMinutes) {
                throw new Error('Booking data is missing. Choose a slot again.');
            }

            if (!currentTonConnectUi) {
                throw new Error('TON Connect is not initialized.');
            }

            if (!currentWalletAddress) {
                throw new Error('TON wallet is not connected.');
            }

            const actualChain = String(currentTonConnectUi?.wallet?.account?.chain || '');

            if (actualChain !== ACTIVE_TON_CHAIN) {
                throw new Error(
                    `Wrong wallet network. Expected ${ACTIVE_TON_NETWORK_LABEL}, got ${describeTonChain(actualChain)} (${actualChain}).`
                );
            }

            setDebugStatus('Creating booking…');

            const requested = await createBookingRequest();

            if (!requested?.id) {
                throw new Error('Booking was not created: missing booking id.');
            }

            setDebugStatus('Preparing TON escrow contract…');

            const startedRaw = await beginBookingPayment(requested.id);
            const started = normalizePaymentPayload(startedRaw);
            currentPayment = started;

            debugBooking('payment payload', {

                booking_id: started.booking_id,

                payment_id: started.payment_id,

                contract: started.contract_address,

                amount: started.amount_nano_ton,

                deploy: started.total_deploy_value_nano_ton,

                hasStateInit: !!started.state_init_boc
            });

            if (!started?.contract_address && !started?.destination_address) {
                throw new Error('TON Worker did not return contract address.');
            }

            if (!started?.amount_nano_ton) {
                throw new Error('TON Worker did not return payment amount.');
            }

            if (!started?.state_init_boc) {
                throw new Error('TON Worker did not return state_init_boc. Escrow contract deployment needs it.');
            }

            setDebugStatus('Open Telegram Wallet and approve the escrow transaction…');

            const txResult = await sendTonBookingPayment(started);

            if (!txResult) {
                throw new Error('Wallet returned without transaction result.');
            }

            try {
                await showReturnedFromWalletAndConfirmPayment(requested.id, txResult);
            } catch (error) {

                debugBooking('booking flow aborted', {
                    message: error?.message
                });

                return;
            }
        } catch (error) {
            debugBooking('booking/payment flow ERROR', {
                message: error?.message,
                stack: error?.stack
            });

            setDebugStatus(error.message || 'Booking failed.');

            if (
                els.paymentCheckingModal &&
                !els.paymentCheckingModal.classList.contains('hidden')
            ) {
                setPaymentCheckingDetail(
                    error.message || 'Payment confirmation failed. You can close this window and check the booking later.'
                );

                showPaymentCheckingCloseButton();
            }
        } finally {
            els.bookingConfirmSubmit.disabled = false;
        }
    });

    els.paymentConfirmedOk?.addEventListener('click', (event) => {
        event.preventDefault();
        event.stopPropagation();

        closeTelegramMiniAppOrGoHome();
    });

    els.telegramRequiredClose?.addEventListener('click', (event) => {
        event.preventDefault();
        closeModal(els.telegramRequiredModal);
    });

    els.walletRequiredClose?.addEventListener('click', (event) => {
        event.preventDefault();
        closeModal(els.walletRequiredModal);
    });

    els.bookingConfirmClose?.addEventListener('click', (event) => {
        event.preventDefault();
        closeModal(els.bookingConfirmModal);
    });

    els.prevPeriodBtn?.addEventListener('click', (event) => {
        event.preventDefault();
        currentOffsetDays = Math.max(0, currentOffsetDays - 7);
        loadPublicPage();
    });

    els.nextPeriodBtn?.addEventListener('click', (event) => {
        event.preventDefault();
        currentOffsetDays += 7;
        loadPublicPage();
    });

    els.paymentCheckingClose?.addEventListener('click', (event) => {
        event.preventDefault();
        event.stopPropagation();

        closeModal(els.paymentCheckingModal);
    });

    document.addEventListener('visibilitychange', () => {
        debugBooking('document.visibilitychange', {
            visibilityState: document.visibilityState
        });
    });

    window.addEventListener('focus', () => {
        debugBooking('window.focus');
    });

    window.addEventListener('blur', () => {
        debugBooking('window.blur');
    });

    window.addEventListener('pageshow', () => {
        debugBooking('window.pageshow');
    });

    window.addEventListener('pagehide', () => {
        debugBooking('window.pagehide');
    });
}

async function forceDisconnectWallet() {
    if (!currentTonConnectUi) {
        debugBooking('forceDisconnectWallet: TonConnect not initialized');
        return;
    }

    try {
        debugBooking('forceDisconnectWallet: before disconnect', safeTonConnectUiSnapshot(currentTonConnectUi));

        await currentTonConnectUi.disconnect();

        currentWalletAddress = '';

        debugBooking('forceDisconnectWallet: disconnected');
        setDebugStatus('Wallet disconnected. Reconnect wallet and try again.');
    } catch (error) {
        debugBooking('forceDisconnectWallet: ERROR', {
            message: error?.message,
            stack: error?.stack
        });

        setDebugStatus(error?.message || 'Wallet disconnect failed.');
    }
}

window.forceDisconnectWallet = forceDisconnectWallet;

function boot() {
    bindDom();
    ensurePaymentModals();
    ensureDebugPanel();
    installGlobalErrorLogging();

    debugBooking('expert-public.js loaded', {
        href: window.location.href,
        readyState: document.readyState,
        hasDebugStatus: !!els.debugStatus,
        hasBookingDebugPanel: !!document.getElementById('booking-debug-panel'),
        hasBookBtn: !!els.bookBtn,
        hasConfirmBtn: !!els.bookingConfirmSubmit,
        hasTonConnectUiGlobal: !!window.TON_CONNECT_UI,
        hasPaymentCheckingDetail: !!els.paymentCheckingDetail,
        hasPaymentCheckingClose: !!els.paymentCheckingClose,
        hasPaymentConfirmedModal: !!els.paymentConfirmedModal,
        hasPaymentConfirmedOk: !!els.paymentConfirmedOk,
    });

    initTelegramWebApp();

    currentSlug = getSlugFromPath();

    debugBooking('boot', {
        href: window.location.href,
        origin: window.location.origin,
        slug: currentSlug,
        expectedNetwork: ACTIVE_TON_NETWORK_LABEL,
        expectedChain: ACTIVE_TON_CHAIN,
        importedNetwork: TON_APP_NETWORK,
        bot: BOT,
        isTelegramMiniApp: !!window.Telegram?.WebApp,
        telegramVersion: window.Telegram?.WebApp?.version || null,
        telegramPlatform: window.Telegram?.WebApp?.platform || null,
        resolvedTelegramUser: resolveTelegramUser()
    });

    bindEvents();

    loadPublicPage();
    updateBookingUi();
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot, { once: true });
} else {
    boot();
}