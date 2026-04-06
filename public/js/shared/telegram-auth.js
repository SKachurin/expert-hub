import {
    TELEGRAM_USER_KEY,
    TELEGRAM_USER_TTL_MS
} from './app-config.js';

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

    if (miniUser) {
        return saveTelegramUser(miniUser);
    }

    return getStoredTelegramUser();
}

export function initTelegramWebApp() {
    const tg = window.Telegram?.WebApp;

    if (!tg) return;

    try {
        tg.ready();
        tg.expand();
    } catch (e) {
        console.error('Telegram WebApp init error:', e);
    }
}