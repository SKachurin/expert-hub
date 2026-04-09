import {
    initTelegramWebApp,
    resolveTelegramUser
} from '/js/shared/telegram-auth.js';

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

document.addEventListener('DOMContentLoaded', () => {
    initTelegramWebApp();

    const currentUser = resolveTelegramUser();
    console.log('index resolved telegram user', currentUser);
    console.log('tg start param', getTelegramStartParam());

    if (maybeRedirectFromTelegramStartParam()) {
        return;
    }
});