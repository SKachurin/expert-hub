import { els } from './dom.js';
import { state } from './state.js';
import { bindChipGroup } from './chip-groups.js';
import { connectAnotherGoogleCalendar, consumeGoogleSessionFromUrl, addPendingGoogleSession } from './calendars.js';
import { initTonConnectForEdit } from './wallet.js';
import { resolveTelegramUser, renderProfileCard, bindTelegramWindowAuth } from './telegram.js';
import { loadExpertData, saveExpertData } from './api.js';
import { setDebugStatus } from './form.js';
import { bindDeleteProfileFlow } from './delete-profile.js';

document.addEventListener('DOMContentLoaded', async () => {
    bindChipGroup(els.workingDaysGroup);
    bindChipGroup(els.durationsGroup);
    bindTelegramWindowAuth();

    const tg = window.Telegram?.WebApp;
    if (tg) {
        try {
            tg.ready();
            tg.expand();
        } catch (e) {
            console.error('Telegram WebApp init error on edit page:', e);
        }
    }

    state.currentTelegramUser = resolveTelegramUser();
    renderProfileCard(state.currentTelegramUser);

    if (state.currentTelegramUser?.id) {
        await loadExpertData();
        initTonConnectForEdit();
        bindDeleteProfileFlow();
    } else {
        setDebugStatus('Authorize with Telegram to load this profile.');
    }

    const returnedGoogleSessionId = consumeGoogleSessionFromUrl();
    if (returnedGoogleSessionId) {
        addPendingGoogleSession(returnedGoogleSessionId);
        setDebugStatus('Google calendar connected. Save profile to attach it.');
    }

    els.connectGoogleCalendarBtn?.addEventListener('click', connectAnotherGoogleCalendar);
    els.saveProfileBtn.addEventListener('click', saveExpertData);
});