import { els } from './dom.js';
import { state } from './state.js';
import { escapeHtml, getSlugFromPath } from './utils.js';
import { setDebugStatus } from './form.js';

export function consumeGoogleSessionFromUrl() {
    const url = new URL(window.location.href);
    const sessionId = url.searchParams.get('google_session');

    if (!sessionId) return null;

    url.searchParams.delete('google_session');
    window.history.replaceState({}, '', url.toString());

    return sessionId;
}

export function addPendingGoogleSession(sessionId) {
    if (sessionId && !state.pendingGoogleSessionIds.includes(sessionId)) {
        state.pendingGoogleSessionIds.push(sessionId);
    }
}

export function buildExistingCalendarTitle(item) {
    const providerName = item.provider === 'google' ? 'Google' : (item.provider || 'Calendar');
    const calendarName = item.selected_calendar_name?.trim() || 'Primary calendar';
    return `${providerName} · ${calendarName}`;
}

export function buildExistingCalendarMeta(item) {
    const lines = [];
    if (item.account_email?.trim()) lines.push(item.account_email.trim());
    if (item.selected_calendar_timezone?.trim()) lines.push(item.selected_calendar_timezone.trim());
    return lines;
}

export function buildPrimaryCalendarOptionLabel(item) {
    const providerName = item.provider === 'google' ? 'Google' : (item.provider || 'Calendar');
    const calendarName = item.selected_calendar_name?.trim() || 'Primary calendar';
    return `${providerName} · ${calendarName}`;
}

export function populatePrimaryCalendarOptions(items, selectedId) {
    if (!els.primaryCalendar) return;

    els.primaryCalendar.innerHTML = '';

    if (!Array.isArray(items) || !items.length) {
        const option = document.createElement('option');
        option.value = '';
        option.textContent = 'No connected calendars';
        els.primaryCalendar.appendChild(option);
        return;
    }

    items.forEach((item) => {
        const option = document.createElement('option');
        option.value = String(item.id);
        option.textContent = buildPrimaryCalendarOptionLabel(item);
        option.selected = Number(selectedId) === Number(item.id);
        els.primaryCalendar.appendChild(option);
    });
}

export function renderExistingCalendarConnections() {
    if (!els.existingCalendarList) return;

    els.existingCalendarList.innerHTML = '';

    if (!state.existingCalendarConnections.length) {
        els.existingCalendarList.innerHTML = '<div class="section-note">No connected calendars yet.</div>';
        return;
    }

    state.existingCalendarConnections.forEach((item) => {
        const card = document.createElement('div');
        card.className = 'existing-calendar-card';

        const metaLines = buildExistingCalendarMeta(item)
            .map((line) => `<div class="existing-calendar-subline">${escapeHtml(line)}</div>`)
            .join('');

        card.innerHTML = `
            <button
                type="button"
                class="existing-calendar-delete"
                data-connection-id="${item.id}"
                aria-label="Delete calendar"
            >×</button>
            <div class="existing-calendar-title">${escapeHtml(buildExistingCalendarTitle(item))}</div>
            ${metaLines}
        `;

        els.existingCalendarList.appendChild(card);
    });

    els.existingCalendarList.querySelectorAll('.existing-calendar-delete').forEach((btn) => {
        btn.addEventListener('click', async () => {
            const slug = getSlugFromPath();
            const connectionId = btn.dataset.connectionId;

            try {
                btn.disabled = true;

                const response = await fetch(
                    `/api/experts/${encodeURIComponent(slug)}/calendar-connections/${encodeURIComponent(connectionId)}`,
                    { method: 'DELETE' }
                );

                if (!response.ok) {
                    const text = await response.text().catch(() => '');
                    throw new Error(`${response.status} ${text}`);
                }

                state.existingCalendarConnections = state.existingCalendarConnections.filter(
                    (item) => String(item.id) !== String(connectionId)
                );

                renderExistingCalendarConnections();
                populatePrimaryCalendarOptions(state.existingCalendarConnections, null);
                setDebugStatus('Calendar deleted.');
            } catch (error) {
                console.error(error);
                btn.disabled = false;
                setDebugStatus(`Delete failed: ${error.message}`);
            }
        });
    });
}

export function connectAnotherGoogleCalendar() {
    if (!state.currentTelegramUser?.id) {
        setDebugStatus('Connect Telegram first.');
        return;
    }

    window.location.href =
        `/oauth/google/start?telegram_id=${encodeURIComponent(state.currentTelegramUser.id)}&return_to=expert_edit&slug=${encodeURIComponent(getSlugFromPath())}`;
}