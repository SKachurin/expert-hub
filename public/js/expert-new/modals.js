export function focusTelegramConnection() {
    const profileCard = document.getElementById('profile-card');
    if (!profileCard) return;

    profileCard.scrollIntoView({ behavior: 'smooth', block: 'center' });

    setTimeout(() => {
        const openLink = profileCard.querySelector('.telegram-open-link');
        const loginSlot = profileCard.querySelector('#telegram-login-slot iframe');

        if (openLink) {
            openLink.focus();
            return;
        }

        if (loginSlot) {
            loginSlot.focus();
        }
    }, 220);
}

export function focusCalendarProviderField() {
    const select =
        document.getElementById('calendar-provider-1') ||
        document.querySelector('[id^="calendar-provider-"]');

    if (!select) return;

    select.scrollIntoView({ behavior: 'smooth', block: 'center' });

    setTimeout(() => {
        select.focus();
    }, 220);
}

export function showCalendarProviderModal(message) {
    const overlay = document.getElementById('calendar-provider-modal');
    const text = document.getElementById('calendar-provider-modal-text');
    const okBtn = document.getElementById('calendar-provider-modal-ok');

    if (!overlay || !text || !okBtn) {
        alert(message || 'Choose calendar provider first.');
        focusCalendarProviderField();
        return;
    }

    text.textContent = message || 'Choose calendar provider first.';
    overlay.hidden = false;

    requestAnimationFrame(() => {
        okBtn.focus();
    });
}

export function hideCalendarProviderModal() {
    const overlay = document.getElementById('calendar-provider-modal');
    if (!overlay) return;
    overlay.hidden = true;
}

export function showTelegramRequiredModal(message) {
    const overlay = document.getElementById('telegram-required-modal');
    const text = document.getElementById('telegram-required-modal-text');
    const okBtn = document.getElementById('telegram-required-modal-ok');

    if (!overlay || !text || !okBtn) {
        alert(message || 'Connect Telegram first.');
        focusTelegramConnection();
        return;
    }

    text.textContent = message || 'Connect Telegram first.';
    overlay.hidden = false;

    requestAnimationFrame(() => {
        okBtn.focus();
    });
}

export function hideTelegramRequiredModal() {
    const overlay = document.getElementById('telegram-required-modal');
    if (!overlay) return;
    overlay.hidden = true;
}

export function showGoogleScopeDeniedModal(message) {
    const overlay = document.getElementById('google-scope-denied-modal');
    const text = document.getElementById('google-scope-denied-modal-text');
    const okBtn = document.getElementById('google-scope-denied-modal-ok');

    if (!overlay || !text || !okBtn) {
        alert(message || 'Google Calendar permission was not granted.');
        focusCalendarProviderField();
        return;
    }

    text.textContent = message || 'Google Calendar permission was not granted.';
    overlay.hidden = false;

    requestAnimationFrame(() => {
        okBtn.focus();
    });
}

export function hideGoogleScopeDeniedModal() {
    const overlay = document.getElementById('google-scope-denied-modal');
    if (!overlay) return;
    overlay.hidden = true;
}

export function handleGoogleErrorFromUrl() {
    const url = new URL(window.location.href);
    const googleError = url.searchParams.get('google_error');

    if (!googleError) return;

    if (googleError === 'calendar_scope_denied') {
        showGoogleScopeDeniedModal(
            'Google signed you in, but Calendar permission was not granted. Please try again and allow Calendar access.'
        );
    } else if (googleError === 'oauth_error') {
        showGoogleScopeDeniedModal('Google authorization was cancelled or denied.');
    } else if (googleError === 'token_exchange_failed') {
        showGoogleScopeDeniedModal('Google sign-in failed during token exchange. Please try again.');
    } else if (googleError === 'userinfo_failed') {
        showGoogleScopeDeniedModal('Google sign-in succeeded, but account info could not be loaded.');
    } else if (googleError === 'calendar_list_failed') {
        showGoogleScopeDeniedModal('Google sign-in succeeded, but calendar list could not be loaded.');
    }

    url.searchParams.delete('google_error');
    url.searchParams.delete('google_scope');
    url.searchParams.delete('google_error_message');
    window.history.replaceState({}, '', url.toString());
}

export function initModals() {
    document.getElementById('calendar-provider-modal-ok')?.addEventListener('click', () => {
        hideCalendarProviderModal();
        focusCalendarProviderField();
    });

    document.getElementById('calendar-provider-modal')?.addEventListener('click', (event) => {
        if (event.target.id === 'calendar-provider-modal') {
            hideCalendarProviderModal();
            focusCalendarProviderField();
        }
    });

    document.getElementById('telegram-required-modal-ok')?.addEventListener('click', () => {
        hideTelegramRequiredModal();
        focusTelegramConnection();
    });

    document.getElementById('telegram-required-modal')?.addEventListener('click', (event) => {
        if (event.target.id === 'telegram-required-modal') {
            hideTelegramRequiredModal();
            focusTelegramConnection();
        }
    });

    document.getElementById('google-scope-denied-modal-ok')?.addEventListener('click', () => {
        hideGoogleScopeDeniedModal();
        focusCalendarProviderField();
    });

    document.getElementById('google-scope-denied-modal')?.addEventListener('click', (event) => {
        if (event.target.id === 'google-scope-denied-modal') {
            hideGoogleScopeDeniedModal();
            focusCalendarProviderField();
        }
    });
}