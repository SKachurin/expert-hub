import { BOT, MANIFEST_URL } from '../shared/app-config.js';
import {
    getStoredTelegramUser,
    initTelegramWebApp,
    resolveTelegramUser,
    saveTelegramUser
} from '../shared/telegram-auth.js';
import { fullName } from '../shared/dom-utils.js';

import { els } from './dom.js';
import { expertDraft, loadExpertDraft, saveExpertDraft } from './expert-draft.js';
import { calendarDraft, ensureCalendarDraftBlock, loadCalendarDraft, saveCalendarDraft } from './calendar-draft.js';
import {
    initModals,
    handleGoogleErrorFromUrl
} from './modals.js';
import {
    renderProfileCard,
    setDebugStatus,
    syncExpertDraftToForm,
    updateRegisterButtonState as renderRegisterButtonState,
    updateWalletUi,
    validateProfileDraft
} from './ui.js';
import {
    bindCalendarBlockEvents,
    initAddCalendarCard,
    maybeResumeGoogleSessionFromUrl,
    rebuildCalendarBlocksFromDraft,
    syncGoogleSessionsWithBackend,
    updateAddCalendarCardVisibility,
    updateCalendarBadge,
    updateCalendarConnectButtonByNumber
} from './calendar.js';
import { registerExpert } from './register.js';

let currentTelegramUser = null;
let currentWallet = null;
let tonUi = null;

function getCurrentTelegramUser() {
    return currentTelegramUser || getStoredTelegramUser();
}

function setCurrentWallet(wallet) {
    currentWallet = wallet;
}

function updateRegisterButtonState() {
    renderRegisterButtonState(currentTelegramUser, currentWallet);
}

function updateDraftFromInputs() {
    expertDraft.display_name = els.displayName.value.trim();
    expertDraft.telegram_bio = els.telegramBio.value.trim();
    expertDraft.timezone = els.timezone.value.trim();
    expertDraft.hourly_rate = els.hourlyRate.value.trim() || '1.00';
    expertDraft.currency = els.currency.value.trim() || 'USD';
    expertDraft.work_start_time = els.workStart.value.trim();
    expertDraft.work_end_time = els.workEnd.value.trim();

    saveExpertDraft();
    validateProfileDraft(updateRegisterButtonState);
}

function initFormDefaults() {
    if (!expertDraft.timezone) {
        expertDraft.timezone = Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
    }

    if (!expertDraft.display_name && currentTelegramUser) {
        expertDraft.display_name = fullName(currentTelegramUser);
    }

    syncExpertDraftToForm();

    ensureCalendarDraftBlock(1);
    updateCalendarBadge();
    saveExpertDraft();
    saveCalendarDraft();
    validateProfileDraft(updateRegisterButtonState);
}

async function initTelegramState() {
    initTelegramWebApp();

    const resolvedUser = resolveTelegramUser();

    if (resolvedUser?.id) {
        currentTelegramUser = resolvedUser;
        renderProfileCard(currentTelegramUser, BOT);
        updateRegisterButtonState();
        setDebugStatus('Telegram user restored.');
        return;
    }

    renderProfileCard(null, BOT);
    updateRegisterButtonState();
    setDebugStatus('Web fallback mode.');
}

async function initTonConnect() {
    tonUi = new TON_CONNECT_UI.TonConnectUI({
        manifestUrl: MANIFEST_URL,
        buttonRootId: 'ton-connect'
    });

    await tonUi.connectionRestored;

    updateWalletUi(
        tonUi.wallet,
        setCurrentWallet,
        updateRegisterButtonState
    );

    tonUi.onStatusChange(async (wallet) => {
        updateWalletUi(
            wallet,
            setCurrentWallet,
            updateRegisterButtonState
        );

        if (!wallet) {
            setDebugStatus('Wallet disconnected.');
            return;
        }

        setDebugStatus('Wallet connected.');
    });
}

function initTelegramWindowMessageHandler() {
    window.addEventListener('message', async (e) => {
        if (e.origin !== 'https://oauth.telegram.org') return;

        let payload = e.data;

        if (typeof payload === 'string') {
            try {
                payload = JSON.parse(payload);
            } catch (err) {
                console.error('Failed to parse Telegram payload', err);
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
            currentTelegramUser = user;
            saveTelegramUser(user);
            renderProfileCard(user, BOT);

            if (!els.displayName.value.trim()) {
                expertDraft.display_name = fullName(user);
                els.displayName.value = expertDraft.display_name;
                saveExpertDraft();
            }

            updateRegisterButtonState();
            setDebugStatus('Telegram connected on web.');
        }
    });
}

function initFormListeners() {
    [
        els.displayName,
        els.telegramBio,
        els.timezone,
        els.hourlyRate,
        els.currency,
        els.workStart,
        els.workEnd
    ].forEach((el) => {
        el.addEventListener('input', updateDraftFromInputs);
        el.addEventListener('change', updateDraftFromInputs);
    });

    els.workingDaysGroup.querySelectorAll('.check-chip').forEach((chip) => {
        chip.addEventListener('click', () => {
            const input = chip.querySelector('input');
            input.checked = !input.checked;
            chip.classList.toggle('active', input.checked);

            expertDraft.working_days = Array.from(
                els.workingDaysGroup.querySelectorAll('input:checked')
            ).map((el) => el.value);

            saveExpertDraft();
            validateProfileDraft(updateRegisterButtonState);
        });
    });

    els.durationsGroup.querySelectorAll('.check-chip').forEach((chip) => {
        chip.addEventListener('click', () => {
            const input = chip.querySelector('input');
            input.checked = !input.checked;
            chip.classList.toggle('active', input.checked);

            expertDraft.allowed_session_durations = Array.from(
                els.durationsGroup.querySelectorAll('input:checked')
            ).map((el) => Number(el.value));

            saveExpertDraft();
            validateProfileDraft(updateRegisterButtonState);
        });
    });

    els.registerExpertBtn.addEventListener('click', async () => {
        if (els.registerExpertBtn.disabled) return;

        try {
            setDebugStatus('Registering expert...');
            const saved = await registerExpert(currentTelegramUser, currentWallet);

            setDebugStatus('Expert saved. Redirecting...');

            const successUrl = new URL('/created.html', window.location.origin);
            successUrl.searchParams.set('slug', saved.public_slug);

            window.location.href = successUrl.toString();
        } catch (err) {
            console.error(err);
            setDebugStatus(`Registration error: ${err.message}`);
        }
    });
}

function bindCalendarBlock(number) {
    bindCalendarBlockEvents(
        number,
        getCurrentTelegramUser,
        updateRegisterButtonState
    );
}

(async () => {
    loadExpertDraft();
    loadCalendarDraft();

    initModals();
    initTelegramWindowMessageHandler();
    initFormListeners();
    initAddCalendarCard(bindCalendarBlock);

    await initTelegramState();
    initFormDefaults();

    rebuildCalendarBlocksFromDraft(bindCalendarBlock);
    bindCalendarBlock(1);

    for (let number = 1; number <= calendarDraft.blocks.length; number++) {
        updateCalendarConnectButtonByNumber(number);
    }

    await syncGoogleSessionsWithBackend(updateRegisterButtonState);

    updateCalendarBadge();
    updateAddCalendarCardVisibility();
    handleGoogleErrorFromUrl();

    try {
        await maybeResumeGoogleSessionFromUrl(updateRegisterButtonState);
    } catch (err) {
        console.error(err);
        setDebugStatus(`Google Calendar connection error: ${err.message}`);
        updateCalendarConnectButtonByNumber(1);
        updateCalendarBadge();
        updateAddCalendarCardVisibility();
    }

    await initTonConnect();
    updateRegisterButtonState();
})();