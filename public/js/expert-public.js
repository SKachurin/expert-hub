import {
    initTelegramWebApp,
    resolveTelegramUser
} from '/js/shared/telegram-auth.js';

const debugStatusEl = document.getElementById('debug-status');

const expertNameEl = document.getElementById('expert-name');
const expertHeadlineEl = document.getElementById('expert-headline');
const expertAvatarEl = document.getElementById('expert-avatar');
const expertAvatarPlaceholderEl = document.getElementById('expert-avatar-placeholder');
const expertUsernameEl = document.getElementById('expert-username');
const expertTimezoneEl = document.getElementById('expert-timezone');
const expertRatingEl = document.getElementById('expert-rating');
const expertRateEl = document.getElementById('expert-rate');
const expertDurationsEl = document.getElementById('expert-durations');
const expertBioEl = document.getElementById('expert-bio');

const slotsRangeLabelEl = document.getElementById('slots-range-label');
const slotsListEl = document.getElementById('slots-list');
const slotsEmptyEl = document.getElementById('slots-empty');
const reviewsListEl = document.getElementById('reviews-list');
const reviewsEmptyEl = document.getElementById('reviews-empty');

const prevPeriodBtnEl = document.getElementById('prev-period-btn');
const nextPeriodBtnEl = document.getElementById('next-period-btn');

const durationPickerWrapEl = document.getElementById('duration-picker-wrap');
const durationPickerEl = document.getElementById('duration-picker');
const editProfileBtnEl = document.getElementById('edit-profile-btn');

const bookBtnEl = document.getElementById('book-btn');
const selectedSlotSummaryEl = document.getElementById('selected-slot-summary');

const telegramRequiredModalEl = document.getElementById('telegram-required-modal');
const walletRequiredModalEl = document.getElementById('wallet-required-modal');
const bookingConfirmModalEl = document.getElementById('booking-confirm-modal');

const telegramRequiredCloseEl = document.getElementById('telegram-required-close');
const walletRequiredCloseEl = document.getElementById('wallet-required-close');
const bookingConfirmCloseEl = document.getElementById('booking-confirm-close');
const bookingConfirmSubmitEl = document.getElementById('booking-confirm-submit');
const bookingConfirmTextEl = document.getElementById('booking-confirm-text');

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

function cancelCurrentPageRequest() {
    if (currentPageRequestController) {
        currentPageRequestController.abort();
        currentPageRequestController = null;
    }
}

function updatePeriodButtonsState(isLoading = false) {
    if (!prevPeriodBtnEl || !nextPeriodBtnEl) return;

    prevPeriodBtnEl.disabled = isLoading || currentOffsetDays <= 0;
    nextPeriodBtnEl.disabled = isLoading;
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
        if (stillExists) break;
    }

    if (!stillExists) {
        selectedSlot = null;
    }
}

function setDebugStatus(text) {
    debugStatusEl.textContent = text || '';
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

function renderExpert(data) {
    expertNameEl.textContent = data.display_name || 'Expert';
    expertHeadlineEl.textContent = data.telegram_bio || 'Expert profile';
    expertUsernameEl.textContent = data.username ? `@${data.username}` : 'No public username';
    expertTimezoneEl.textContent = `Timezone: ${data.timezone}`;
    expertRatingEl.textContent = data.reviews_count > 0
        ? `${data.expert_rating}/5 · ${data.reviews_count} review(s)`
        : 'No reviews yet';

    expertRateEl.textContent = formatMoney(data.hourly_rate, data.currency);
    expertDurationsEl.textContent = formatDurations(data.allowed_session_durations);
    expertBioEl.textContent = data.telegram_bio || 'No description yet.';

    expertAvatarPlaceholderEl.textContent = (data.display_name || 'E').trim().charAt(0).toUpperCase() || 'E';

    if (data.photo_url) {
        expertAvatarEl.src = data.photo_url;
        expertAvatarEl.classList.remove('hidden');
        expertAvatarPlaceholderEl.classList.add('hidden');
    } else {
        expertAvatarEl.classList.add('hidden');
        expertAvatarPlaceholderEl.classList.remove('hidden');
    }
    updateEditProfileButton(data);
}

function renderSlots(payload) {
    slotsRangeLabelEl.textContent = formatDateRangeLabel(payload.period_start, payload.period_end);
    slotsListEl.innerHTML = '';

    if (!Array.isArray(payload.days) || !payload.days.length) {
        slotsEmptyEl.classList.remove('hidden');
        selectedSlot = null;
        updateBookingUi();
        return;
    }

    const visibleDays = payload.days.filter((day) => Array.isArray(day.slots) && day.slots.length > 0);

    if (!visibleDays.length) {
        slotsEmptyEl.classList.remove('hidden');
        selectedSlot = null;
        updateBookingUi();
        return;
    }

    slotsEmptyEl.classList.add('hidden');

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
                renderSlots(payload);
                updateBookingUi();
            });

            chipList.appendChild(chip);
        });

        card.appendChild(title);
        card.appendChild(chipList);
        slotsListEl.appendChild(card);
    });
}

function renderReviews(reviews) {
    reviewsListEl.innerHTML = '';

    if (!Array.isArray(reviews) || !reviews.length) {
        reviewsEmptyEl.classList.remove('hidden');
        return;
    }

    reviewsEmptyEl.classList.add('hidden');

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

        reviewsListEl.appendChild(card);
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
            slotsListEl.innerHTML = '';
            slotsRangeLabelEl.textContent = '—';
            slotsEmptyEl.classList.remove('hidden');
            selectedSlot = null;
            updateBookingUi();
        }

        setDebugStatus('');
    } catch (error) {
        if (error.name === 'AbortError') {
            return;
        }

        console.error(error);

        if (requestId !== currentPageRequestId) {
            return;
        }

        setDebugStatus(error.message || 'Failed to load public page.');
    } finally {
        if (requestId === currentPageRequestId) {
            currentPageRequestController = null;
            updatePeriodButtonsState(false);
        }
    }
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

function renderDurationPicker(durations) {
    durationPickerEl.innerHTML = '';

    if (!Array.isArray(durations) || !durations.length) {
        durationPickerWrapEl.classList.add('hidden');
        return;
    }

    durationPickerWrapEl.classList.remove('hidden');

    durations.forEach((duration) => {
        const chip = document.createElement('button');
        chip.type = 'button';
        chip.className = 'duration-chip';
        chip.textContent = `${duration} min`;

        if (duration === currentDurationMinutes) {
            chip.classList.add('active');
        }

        chip.addEventListener('click', async () => {
            if (duration === currentDurationMinutes) {
                return;
            }

            currentDurationMinutes = duration;
            renderDurationPicker(expertAllowedDurations);
            await loadAvailabilityOnly();
        });

        durationPickerEl.appendChild(chip);
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

        console.error(error);

        if (requestId !== currentPageRequestId) {
            return;
        }

        setDebugStatus(error.message || 'Failed to load availability.');
    } finally {
        if (requestId === currentPageRequestId) {
            currentPageRequestController = null;
            updatePeriodButtonsState(false);
        }
    }
}

function updateEditProfileButton(data) {
    if (!editProfileBtnEl) {
        return;
    }

    editProfileBtnEl.href = `/e/${encodeURIComponent(data.public_slug)}/edit`;

    const telegramUser = resolveTelegramUser();

    const isOwner =
        telegramUser &&
        Number(telegramUser.id) > 0 &&
        Number(data.telegram_id) === Number(telegramUser.id);

    if (isOwner) {
        editProfileBtnEl.classList.remove('hidden');
    } else {
        editProfileBtnEl.classList.add('hidden');
    }
}


function closeModal(el) {
    el?.classList.add('hidden');
}

function openModal(el) {
    el?.classList.remove('hidden');
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
    if (!bookBtnEl) return;

    const enabled = !!selectedSlot && !!currentDurationMinutes && !!currentExpert;
    bookBtnEl.disabled = !enabled;

    if (!enabled) {
        selectedSlotSummaryEl?.classList.add('hidden');
        selectedSlotSummaryEl.textContent = '';
        return;
    }

    const quote = deriveQuote(currentExpert, currentDurationMinutes);
    selectedSlotSummaryEl?.classList.remove('hidden');
    selectedSlotSummaryEl.textContent =
        `Selected: ${formatSlotLabel(selectedSlot)} · ${currentDurationMinutes} min · ${quote} ${currentExpert.currency}`;
}

function initBookingTonConnect() {
    if (!window.TON_CONNECT_UI || currentTonConnectUi) {
        return;
    }

    currentTonConnectUi = new window.TON_CONNECT_UI.TonConnectUI({
        manifestUrl: `${window.location.origin}/tonconnect-manifest.json`,
        buttonRootId: 'booking-ton-connect'
    });

    if (currentTonConnectUi.wallet?.account?.address) {
        currentWalletAddress = currentTonConnectUi.wallet.account.address;
    }

    currentTonConnectUi.onStatusChange((wallet) => {
        currentWalletAddress = wallet?.account?.address || '';
        if (currentWalletAddress && walletRequiredModalEl && !walletRequiredModalEl.classList.contains('hidden')) {
            closeModal(walletRequiredModalEl);
            openBookingConfirmModal();
        }
    });
}

function openBookingConfirmModal() {
    if (!selectedSlot || !currentExpert) return;

    const quote = deriveQuote(currentExpert, currentDurationMinutes);
    bookingConfirmTextEl.textContent =
        `You are booking ${currentExpert.display_name} for ${currentDurationMinutes} minutes at ${selectedSlot.day_label} ${selectedSlot.start_local}. Prepayment: ${quote} ${currentExpert.currency}.`;
    openModal(bookingConfirmModalEl);
}

async function createBookingRequest() {
    const telegramUser = resolveTelegramUser();
    if (!telegramUser?.id) {
        openModal(telegramRequiredModalEl);
        return;
    }

    const response = await fetch('/api/bookings/request', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            expert_slug: currentSlug,
            slot_start: selectedSlot.start_utc,
            duration_minutes: currentDurationMinutes,
            requested_by_telegram_id: Number(telegramUser.id),
            requested_by_username: telegramUser.username || null,
            requested_by_display_name: [telegramUser.first_name, telegramUser.last_name].filter(Boolean).join(' ') || telegramUser.first_name || 'Telegram user',
            requested_by_ton_wallet: currentWalletAddress || null
        })
    });

    const data = await response.json().catch(() => ({}));
    if (!response.ok) {
        throw new Error(data.error || 'Booking request failed.');
    }

    return data;
}

async function beginBookingPayment(bookingId) {
    const telegramUser = resolveTelegramUser();

    const response = await fetch(`/api/bookings/${encodeURIComponent(bookingId)}/begin-payment`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            telegram_id: Number(telegramUser.id),
            ton_wallet_customer: currentWalletAddress
        })
    });

    const data = await response.json().catch(() => ({}));
    if (!response.ok) {
        throw new Error(data.error || 'Payment start failed.');
    }
    return data;
}

bookBtnEl?.addEventListener('click', async () => {
    try {
        const telegramUser = resolveTelegramUser();
        if (!telegramUser?.id) {
            openModal(telegramRequiredModalEl);
            return;
        }

        initBookingTonConnect();

        if (!currentWalletAddress) {
            openModal(walletRequiredModalEl);
            return;
        }

        openBookingConfirmModal();
    } catch (error) {
        console.error(error);
        setDebugStatus(error.message || 'Booking flow failed.');
    }
});

bookingConfirmSubmitEl?.addEventListener('click', async () => {
    try {
        bookingConfirmSubmitEl.disabled = true;

        const requested = await createBookingRequest();
        if (!requested?.id) {
            return;
        }
        const started = await beginBookingPayment(requested.id);

        closeModal(bookingConfirmModalEl);
        setDebugStatus(`Booking #${started.id} created. Status: ${started.status}. Payment: ${started.payment_status || 'n/a'}.`);

        selectedSlot = null;
        updateBookingUi();
        loadPublicPage();
    } catch (error) {
        console.error(error);
        setDebugStatus(error.message || 'Booking failed.');
    } finally {
        bookingConfirmSubmitEl.disabled = false;
    }
});

telegramRequiredCloseEl?.addEventListener('click', () => closeModal(telegramRequiredModalEl));
walletRequiredCloseEl?.addEventListener('click', () => closeModal(walletRequiredModalEl));
bookingConfirmCloseEl?.addEventListener('click', () => closeModal(bookingConfirmModalEl));

document.addEventListener('DOMContentLoaded', () => {
    initTelegramWebApp();

    currentSlug = getSlugFromPath();

    prevPeriodBtnEl.addEventListener('click', () => {
        currentOffsetDays = Math.max(0, currentOffsetDays - 7);
        loadPublicPage();
    });

    nextPeriodBtnEl.addEventListener('click', () => {
        currentOffsetDays += 7;
        loadPublicPage();
    });

    loadPublicPage();
    initBookingTonConnect();
    updateBookingUi();
});