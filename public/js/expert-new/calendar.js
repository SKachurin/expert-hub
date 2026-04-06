import {
    MAX_CALENDAR_BLOCKS
} from '../shared/app-config.js';
import { calendarDraft, ensureCalendarDraftBlock, saveCalendarDraft } from './calendar-draft.js';
import { setBadgeState, setStepState } from './ui.js';
import { els } from './dom.js';
import {
    showCalendarProviderModal,
    showTelegramRequiredModal
} from './modals.js';

export function getCalendarBlocks() {
    return document.querySelectorAll('.calendar-card[data-calendar-block="true"]');
}

export function getNextCalendarBlockNumber() {
    return getCalendarBlocks().length + 1;
}

export function canAddAnotherCalendarBlock() {
    return getCalendarBlocks().length < MAX_CALENDAR_BLOCKS;
}

export function getSelectedCalendarNames(block) {
    if (!block || !Array.isArray(block.google_calendars) || !Array.isArray(block.selected_calendar_ids)) {
        return [];
    }

    return block.google_calendars
        .filter(calendar => block.selected_calendar_ids.includes(calendar.id))
        .map(calendar => calendar.primary ? 'Primary calendar' : (calendar.summary || 'Unnamed calendar'));
}

export function updateCalendarBadge() {
    const connectedCount = calendarDraft.blocks.filter(block => block && block.connected).length;

    setBadgeState(
        els.calendarBadge,
        connectedCount > 0,
        connectedCount === 1 ? '1 connected' : `${connectedCount} connected`,
        'Not connected'
    );

    setStepState(
        els.stepCalendar,
        connectedCount > 0 ? 'Connected' : 'Pending',
        connectedCount > 0
    );
}

export function updateAddCalendarCardVisibility() {
    const addCard = document.getElementById('add-calendar-card');
    if (!addCard) return;

    const hasConnected = calendarDraft.blocks.some(block => block && block.connected);
    const canAdd = canAddAnotherCalendarBlock();

    addCard.style.display = hasConnected && canAdd ? 'flex' : 'none';
}

export function updateCalendarConnectButtonByNumber(number) {
    const providerEl = document.getElementById(`calendar-provider-${number}`);
    const btn = document.getElementById(`calendar-connect-btn-${number}`);
    const btnText = document.getElementById(`calendar-btn-text-${number}`);
    const title = document.getElementById(`calendar-title-${number}`);
    const note = document.getElementById(`calendar-note-${number}`);

    if (!providerEl || !btn || !btnText || !title || !note) {
        return;
    }

    const block = calendarDraft.blocks[number - 1] || {};
    const provider = String(providerEl.value || block.provider || '').toLowerCase();
    const connected = !!block.connected;

    if (providerEl.value !== provider && provider) {
        providerEl.value = provider;
    }

    btn.classList.remove('is-google-pending', 'is-connected', 'show-google-logo');

    if (!provider) {
        title.textContent = 'No provider selected';
        note.textContent = 'Choose Google Calendar or Calendly above.';
        btn.classList.add('is-google-pending');
        btnText.textContent = 'Connect Calendar';
        return;
    }

    if (connected) {
        if (provider === 'google') {
            const selectedNames = getSelectedCalendarNames(block);

            title.textContent = 'Connected calendars';
            note.textContent =
                `${block.google_account_email ? `Connected as ${block.google_account_email}.\n` : ''}` +
                (selectedNames.length ? `Calendars: ${selectedNames.join(', ')}` : 'Connected.');

            btn.classList.add('is-connected');
            btnText.textContent = 'Edit Connected Google Calendar';
            return;
        }

        btnText.textContent = 'Calendar connected';
        return;
    }

    if (provider === 'google') {
        title.textContent = 'Google Calendar';
        note.textContent = 'Connect your Google account and choose up to 2 calendars.';
        btn.classList.add('show-google-logo');
        btnText.textContent = 'Connect Google Calendar';
        return;
    }

    if (provider === 'calendly') {
        title.textContent = 'Calendly';
        note.textContent = 'Calendly is not connected yet. Choose Google Calendar for now.';
        btnText.textContent = 'Connect Calendly';
        return;
    }

    title.textContent = 'Unknown provider';
    note.textContent = 'Provider selected, but not connected yet.';
    btnText.textContent = 'Connect Calendar';
}

export function createCalendarBlock(number, bindCalendarBlockEvents) {
    const firstBlock = document.getElementById('calendar-card-1');
    const addCard = document.getElementById('add-calendar-card');

    if (!firstBlock || !addCard) {
        console.error('Calendar block template or add card not found');
        return null;
    }

    const clone = firstBlock.cloneNode(true);

    clone.id = `calendar-card-${number}`;
    clone.setAttribute('data-calendar-number', String(number));
    clone.setAttribute('data-calendar-block', 'true');

    const sectionHead = clone.querySelector('.section-head');
    if (sectionHead) {
        const badge = sectionHead.querySelector('#calendar-badge');
        if (badge) badge.remove();
    }

    const providerLabel = clone.querySelector('label[for="calendar-provider-1"]');
    if (providerLabel) providerLabel.setAttribute('for', `calendar-provider-${number}`);

    const provider = clone.querySelector('#calendar-provider-1');
    if (provider) {
        provider.id = `calendar-provider-${number}`;
        provider.value = '';
    }

    const title = clone.querySelector('#calendar-title-1');
    if (title) {
        title.id = `calendar-title-${number}`;
        title.textContent = 'No provider selected';
    }

    const note = clone.querySelector('#calendar-note-1');
    if (note) {
        note.id = `calendar-note-${number}`;
        note.textContent = 'Choose Google Calendar or Calendly above.';
    }

    const connectBtn = clone.querySelector('#calendar-connect-btn-1');
    if (connectBtn) {
        connectBtn.id = `calendar-connect-btn-${number}`;
        connectBtn.className = 'calendar-connect-btn';
    }

    const btnLogo = clone.querySelector('#calendar-btn-logo-1');
    if (btnLogo) btnLogo.id = `calendar-btn-logo-${number}`;

    const btnText = clone.querySelector('#calendar-btn-text-1');
    if (btnText) {
        btnText.id = `calendar-btn-text-${number}`;
        btnText.textContent = 'Connect Calendar';
    }

    addCard.parentNode.insertBefore(clone, addCard);

    ensureCalendarDraftBlock(number);
    bindCalendarBlockEvents(number);
    updateCalendarConnectButtonByNumber(number);
    updateAddCalendarCardVisibility();

    clone.scrollIntoView({ behavior: 'smooth', block: 'center' });

    return clone;
}

export function bindCalendarBlockEvents(
    number,
    getCurrentTelegramUser,
    updateRegisterButtonState
) {
    const providerEl = document.getElementById(`calendar-provider-${number}`);
    const connectBtn = document.getElementById(`calendar-connect-btn-${number}`);

    if (!providerEl || !connectBtn) return;

    providerEl.addEventListener('change', () => {
        ensureCalendarDraftBlock(number);

        const block = calendarDraft.blocks[number - 1];
        block.provider = providerEl.value || '';
        block.connected = false;
        block.google_session_id = '';
        block.google_account_email = '';
        block.google_calendars = [];
        block.selected_calendar_ids = [];

        saveCalendarDraft();
        updateCalendarBadge();
        updateCalendarConnectButtonByNumber(number);
        updateRegisterButtonState();
    });

    connectBtn.addEventListener('click', () => {
        ensureCalendarDraftBlock(number);

        const provider = String(providerEl.value || '').toLowerCase();

        if (!provider) {
            showCalendarProviderModal('Choose calendar provider first.');
            return;
        }

        if (provider === 'google') {
            const tgUser = getCurrentTelegramUser();

            if (!tgUser?.id) {
                showTelegramRequiredModal('Connect Telegram first.');
                return;
            }

            sessionStorage.setItem('pending_calendar_block', String(number));
            window.location.href = `/oauth/google/start?telegram_id=${encodeURIComponent(tgUser.id)}`;
            return;
        }

        if (provider === 'calendly') {
            showCalendarProviderModal('Calendly is not connected yet. Choose Google Calendar for now.');
        }
    });
}

export async function loadGoogleSession(sessionId, updateRegisterButtonState) {
    const response = await fetch(`/google/calendars/session/${encodeURIComponent(sessionId)}`);
    if (!response.ok) {
        throw new Error(`Failed to load Google session: ${response.status}`);
    }

    const data = await response.json();

    const selected = prompt(
        `Google account: ${data.account_email || 'unknown'}\n\nSelect up to 2 calendars by numbers, comma-separated:\n\n` +
        data.calendars.map((c, i) => `${i + 1}. ${c.summary}${c.primary ? ' (primary)' : ''}`).join('\n'),
        '1'
    );

    if (selected === null) {
        throw new Error('Google calendar selection cancelled');
    }

    const indexes = selected
        .split(',')
        .map(v => Number(v.trim()))
        .filter(v => Number.isInteger(v) && v >= 1 && v <= data.calendars.length);

    const uniqueIndexes = [...new Set(indexes)].slice(0, 2);

    if (!uniqueIndexes.length) {
        throw new Error('No calendars selected');
    }

    const selectedCalendarIds = uniqueIndexes.map(i => data.calendars[i - 1].id);

    const saveResp = await fetch(`/google/calendars/session/${encodeURIComponent(sessionId)}/select`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ selected_calendar_ids: selectedCalendarIds })
    });

    if (!saveResp.ok) {
        const text = await saveResp.text().catch(() => '');
        throw new Error(`Failed to save Google calendar selection: ${saveResp.status} ${text}`);
    }

    const blockNumber = Number(sessionStorage.getItem('pending_calendar_block') || '1');
    ensureCalendarDraftBlock(blockNumber);

    const block = calendarDraft.blocks[blockNumber - 1];
    block.provider = 'google';
    block.connected = true;
    block.google_session_id = data.session_id;
    block.google_account_email = data.account_email || '';
    block.google_calendars = Array.isArray(data.calendars) ? data.calendars : [];
    block.selected_calendar_ids = selectedCalendarIds;

    const providerEl = document.getElementById(`calendar-provider-${blockNumber}`);
    if (providerEl) {
        providerEl.value = 'google';
    }

    saveCalendarDraft();
    updateCalendarBadge();
    updateCalendarConnectButtonByNumber(blockNumber);
    updateAddCalendarCardVisibility();
    updateRegisterButtonState();

    sessionStorage.removeItem('pending_calendar_block');
}

export function maybeResumeGoogleSessionFromUrl(updateRegisterButtonState) {
    const url = new URL(window.location.href);
    const sessionId = url.searchParams.get('google_session');

    if (!sessionId) return Promise.resolve();

    url.searchParams.delete('google_session');
    window.history.replaceState({}, '', url.toString());

    return loadGoogleSession(sessionId, updateRegisterButtonState);
}

export async function syncGoogleSessionsWithBackend(updateRegisterButtonState) {
    let changed = false;

    for (let index = 0; index < calendarDraft.blocks.length; index++) {
        const block = calendarDraft.blocks[index];

        if (!block || block.provider !== 'google' || !block.google_session_id) {
            continue;
        }

        try {
            const response = await fetch(
                `/google/calendars/session/${encodeURIComponent(block.google_session_id)}`
            );

            if (response.ok) {
                const data = await response.json();

                block.connected = true;
                block.google_account_email = data.account_email || '';
                block.google_calendars = Array.isArray(data.calendars) ? data.calendars : [];
                block.selected_calendar_ids = Array.isArray(data.selected_calendar_ids)
                    ? data.selected_calendar_ids
                    : [];

                continue;
            }

            if (response.status === 404) {
                block.connected = false;
                block.google_session_id = '';
                block.google_account_email = '';
                block.google_calendars = [];
                block.selected_calendar_ids = [];
                changed = true;
                continue;
            }

            console.error(`Google session check failed for block ${index + 1}: ${response.status}`);
        } catch (error) {
            console.error(`Google session check error for block ${index + 1}:`, error);
        }
    }

    if (changed) {
        saveCalendarDraft();
    }

    for (let number = 1; number <= calendarDraft.blocks.length; number++) {
        updateCalendarConnectButtonByNumber(number);
    }

    updateCalendarBadge();
    updateAddCalendarCardVisibility();
    updateRegisterButtonState();
}

export function initAddCalendarCard(bindCalendarBlockEvents) {
    const addCard = document.getElementById('add-calendar-card');
    if (!addCard) return;

    addCard.addEventListener('click', () => {
        if (!canAddAnotherCalendarBlock()) {
            return;
        }

        const nextNumber = getNextCalendarBlockNumber();
        createCalendarBlock(nextNumber, bindCalendarBlockEvents);
    });
}

export function rebuildCalendarBlocksFromDraft(bindCalendarBlockEvents) {
    for (let number = 2; number <= calendarDraft.blocks.length; number++) {
        if (!document.getElementById(`calendar-card-${number}`)) {
            createCalendarBlock(number, bindCalendarBlockEvents);
        }
    }
}