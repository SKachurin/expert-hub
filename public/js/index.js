const APP_HOST = window.location.hostname;
const IS_DEV = APP_HOST === 'dev.experthub.bar';

const TELEGRAM_USER_KEY = IS_DEV
    ? 'Dev_expertHubTelegramUserV1'
    : 'expertHubTelegramUserV1';

const TELEGRAM_USER_TTL_MS = 6 * 60 * 60 * 1000; // 6 hours

function getStoredTelegramUser() {
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

function saveTelegramUser(user) {
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

function resolveTelegramUser() {
    const miniUser = window.Telegram?.WebApp?.initDataUnsafe?.user || null;

    if (miniUser) {
        const user = {
            id: miniUser.id,
            first_name: miniUser.first_name || '',
            last_name: miniUser.last_name || '',
            username: miniUser.username || '',
            photo_url: miniUser.photo_url || ''
        };
        saveTelegramUser(user);
        return user;
    }

    return getStoredTelegramUser();
}

function initIndexTelegramState() {
    const tg = window.Telegram?.WebApp;

    if (tg) {
        try {
            tg.ready();
            tg.expand();
        } catch (e) {
            console.error('Telegram WebApp init error on index:', e);
        }
    }

    resolveTelegramUser();
}

document.addEventListener('DOMContentLoaded', () => {
    initIndexTelegramState();
});