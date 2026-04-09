import {
    initTelegramWebApp,
    resolveTelegramUser
} from '/js/shared/telegram-auth.js';

const popularExpertsListEl = document.getElementById('popular-experts-list');
const popularExpertsEmptyEl = document.getElementById('popular-experts-empty');

function getTelegramStartParam() {
    const url = new URL(window.location.href);

    const fromQuery = url.searchParams.get('tgWebAppStartParam');
    if (fromQuery && fromQuery.trim()) {
        return fromQuery.trim();
    }

    const fromWebApp = window.Telegram?.WebApp?.initDataUnsafe?.start_param;
    if (fromWebApp && String(fromWebApp).trim()) {
        return String(fromWebApp).trim();
    }

    return '';
}

function maybeRedirectFromTelegramStartParam() {
    const startParam = getTelegramStartParam();

    if (!startParam) {
        return false;
    }

    if (startParam === 's') {
        window.location.replace('/expert-new.html');
        return true;
    }

    window.location.replace(`/e/${encodeURIComponent(startParam)}`);
    return true;
}

function escapeHtml(value) {
    return String(value ?? '')
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#039;');
}

function getInitials(name) {
    const safe = String(name || '').trim();
    if (!safe) return 'E';

    const parts = safe.split(/\s+/).filter(Boolean);
    if (parts.length === 1) {
        return parts[0].slice(0, 1).toUpperCase();
    }

    return `${parts[0][0] || ''}${parts[1][0] || ''}`.toUpperCase();
}

function formatRating(value) {
    const num = Number(value);
    if (!Number.isFinite(num)) {
        return 'New';
    }
    return num.toFixed(1);
}

function formatReviewsCount(value) {
    const num = Number(value || 0);
    if (num <= 0) {
        return 'No reviews yet';
    }
    if (num === 1) {
        return '1 review';
    }
    return `${num} reviews`;
}

function renderPopularExperts(items) {
    if (!popularExpertsListEl || !popularExpertsEmptyEl) {
        return;
    }

    popularExpertsListEl.innerHTML = '';

    if (!Array.isArray(items) || !items.length) {
        popularExpertsEmptyEl.classList.remove('hidden');
        return;
    }

    popularExpertsEmptyEl.classList.add('hidden');

    items.forEach((expert) => {
        const card = document.createElement('a');
        card.className = 'popular-expert-tile';
        card.href = `https://t.me/expert_hub_bot?startapp=${encodeURIComponent(expert.public_slug)}`;

        const avatarWrap = document.createElement('div');
        avatarWrap.className = 'popular-expert-avatar-wrap';

        if (expert.photo_url && String(expert.photo_url).trim()) {
            const img = document.createElement('img');
            img.className = 'popular-expert-avatar';
            img.src = expert.photo_url;
            img.alt = expert.display_name || 'Expert avatar';
            avatarWrap.appendChild(img);
        } else {
            const placeholder = document.createElement('div');
            placeholder.className = 'popular-expert-avatar popular-expert-avatar-placeholder';
            placeholder.textContent = getInitials(expert.display_name);
            avatarWrap.appendChild(placeholder);
        }

        const name = document.createElement('div');
        name.className = 'popular-expert-name';
        name.textContent = expert.display_name || 'Expert';

        const username = document.createElement('div');
        username.className = 'popular-expert-username';
        username.textContent = expert.username ? `@${expert.username}` : 'Expert Hub';

        const meta = document.createElement('div');
        meta.className = 'popular-expert-meta';

        const rating = document.createElement('div');
        rating.className = 'popular-expert-rating';
        rating.innerHTML = `<span class="popular-expert-star">★</span><span>${escapeHtml(formatRating(expert.expert_rating))}</span>`;

        const reviews = document.createElement('div');
        reviews.className = 'popular-expert-reviews';
        reviews.textContent = formatReviewsCount(expert.reviews_count);

        meta.appendChild(rating);
        meta.appendChild(reviews);

        card.appendChild(avatarWrap);
        card.appendChild(name);
        card.appendChild(username);
        card.appendChild(meta);

        popularExpertsListEl.appendChild(card);
    });
}

async function loadPopularExperts() {
    if (!popularExpertsListEl || !popularExpertsEmptyEl) {
        return;
    }

    try {
        const response = await fetch('/api/experts/popular?limit=9');
        const payload = await response.json().catch(() => ({ items: [] }));

        if (!response.ok) {
            throw new Error(payload.message || 'Failed to load popular experts.');
        }

        renderPopularExperts(payload.items || []);
    } catch (error) {
        console.error('Failed to load popular experts', error);
        renderPopularExperts([]);
    }
}

document.addEventListener('DOMContentLoaded', () => {
    initTelegramWebApp();

    const currentUser = resolveTelegramUser();
    console.log('index resolved telegram user', currentUser);
    console.log('tg start param', getTelegramStartParam());

    if (maybeRedirectFromTelegramStartParam()) {
        return;
    }

    loadPopularExperts();
});