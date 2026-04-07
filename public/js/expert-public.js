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

let currentOffsetDays = 0;
let currentSlug = null;

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
}

function renderSlots(payload) {
    slotsRangeLabelEl.textContent = formatDateRangeLabel(payload.period_start, payload.period_end);
    slotsListEl.innerHTML = '';

    if (!Array.isArray(payload.days) || !payload.days.length) {
        slotsEmptyEl.classList.remove('hidden');
        return;
    }

    const visibleDays = payload.days.filter((day) => Array.isArray(day.slots) && day.slots.length > 0);

    if (!visibleDays.length) {
        slotsEmptyEl.classList.remove('hidden');
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
            const chip = document.createElement('div');
            chip.className = 'slot-chip';
            chip.textContent = `${slot.start_local} · ${slot.duration_minutes} min`;
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

    try {
        setDebugStatus('Loading public page…');

        const response = await fetch(`/api/experts/${encodeURIComponent(currentSlug)}/public?offset_days=${currentOffsetDays}`);
        if (!response.ok) {
            const text = await response.text().catch(() => '');
            throw new Error(`${response.status} ${text}`);
        }

        const payload = await response.json();

        renderExpert(payload.expert);
        renderSlots(payload.availability);
        renderReviews(payload.reviews);

        setDebugStatus('');
    } catch (error) {
        console.error(error);
        setDebugStatus(`Failed to load expert page: ${error.message}`);
    }
}

document.addEventListener('DOMContentLoaded', () => {
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
});