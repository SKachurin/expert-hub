import { BOT, PUBLIC_ORIGIN, TELEGRAM_USER_KEY, TELEGRAM_USER_TTL_MS, TG_APP_LINK } from './config.js';
import { els } from './dom.js';
import { state } from './state.js';
import { escapeHtml, fullName, initialsFromUser, telegramIconSvg } from './utils.js';
import { loadExpertData } from './api.js';
import { setDebugStatus } from './form.js';

export function getStoredTelegramUser() {
    try {
        const raw = localStorage.getItem(TELEGRAM_USER_KEY);
        if (!raw) return null;

        const parsed = JSON.parse(raw);
        if (!parsed || !parsed.user || !parsed.user.id) {
            localStorage.removeItem(TELEGRAM_USER_KEY);
            return null;
        }

        if (!parsed.expires_at || Date.now() > parsed.expires_at) {
            localStorage.removeItem(TELEGRAM_USER_KEY);
            return null;
        }

        return parsed.user;
    } catch {
        localStorage.removeItem(TELEGRAM_USER_KEY);
        return null;
    }
}

export function saveTelegramUser(user) {
    if (!user || !user.id) return null;

    const storedUser = {
        id: user.id,
        first_name: user.first_name || '',
        last_name: user.last_name || '',
        username: user.username || '',
        photo_url: user.photo_url || ''
    };

    localStorage.setItem(
        TELEGRAM_USER_KEY,
        JSON.stringify({
            user: storedUser,
            expires_at: Date.now() + TELEGRAM_USER_TTL_MS
        })
    );

    return storedUser;
}

export function getMiniAppTelegramUser() {
    const miniUser = window.Telegram?.WebApp?.initDataUnsafe?.user || null;
    if (!miniUser) return null;

    return {
        id: miniUser.id,
        first_name: miniUser.first_name || '',
        last_name: miniUser.last_name || '',
        username: miniUser.username || '',
        photo_url: miniUser.photo_url || ''
    };
}

export function resolveTelegramUser() {
    const miniUser = getMiniAppTelegramUser();
    if (miniUser) return saveTelegramUser(miniUser);
    return getStoredTelegramUser();
}

export function mountTelegramLoginWidget() {
    const hostEl = document.getElementById('telegram-login-slot');
    if (!hostEl) return;

    hostEl.innerHTML = '';

    const iframe = document.createElement('iframe');
    iframe.src = `https://oauth.telegram.org/embed/${BOT}?origin=${encodeURIComponent(PUBLIC_ORIGIN)}&request_access=write&embed=1`;
    iframe.width = 360;
    iframe.height = 54;
    iframe.style.border = '0';
    iframe.style.overflow = 'hidden';
    iframe.setAttribute('scrolling', 'no');

    hostEl.appendChild(iframe);
}

export function renderProfileCard(user) {
    if (!user) {
        els.profileCard.innerHTML = `
            <div class="profile-top">
                <div class="avatar avatar-placeholder">${escapeHtml(initialsFromUser(null))}</div>
                <div class="profile-copy">
                    <div id="telegram-login-slot" class="telegram-login-slot"></div>
                    <div class="profile-sub">Telegram account not connected</div>
                    <a class="telegram-open-link" href="${TG_APP_LINK}" target="_blank" rel="noopener noreferrer">
                        Open in Telegram
                    </a>
                </div>
            </div>
        `;
        mountTelegramLoginWidget();
        els.saveProfileBtn.disabled = true;
        return;
    }

    const photo = user.photo_url
        ? `<img class="avatar" src="${escapeHtml(user.photo_url)}" alt="">`
        : `<div class="avatar avatar-placeholder">${escapeHtml(initialsFromUser(user))}</div>`;

    els.profileCard.innerHTML = `
        <div class="profile-top">
            ${photo}
            <div class="profile-copy">
                <div class="identity-pill">
                    <span class="identity-icon">${telegramIconSvg()}</span>
                    <span>${escapeHtml(fullName(user))}</span>
                </div>
                <div class="profile-sub">Telegram connected</div>
            </div>
        </div>
    `;
}

export function bindTelegramWindowAuth() {
    window.addEventListener('message', async (e) => {
        if (e.origin !== 'https://oauth.telegram.org') return;

        let payload = e.data;
        if (typeof payload === 'string') {
            try {
                payload = JSON.parse(payload);
            } catch {
                return;
            }
        }

        if (payload?.event === 'auth_user' && payload.auth_data) {
            const response = await fetch('/tg-auth', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload.auth_data),
            });

            if (!response.ok) {
                setDebugStatus(`Telegram auth failed: ${response.status}`);
                return;
            }

            const user = await response.json();
            state.currentTelegramUser = saveTelegramUser(user);
            renderProfileCard(state.currentTelegramUser);
            await loadExpertData();
        }
    });
}