import { els } from './dom.js';
import { state } from './state.js';
import { setChipGroupValues, getChipGroupValues } from './chip-groups.js';
import { refreshWalletUi, getWalletAddressForSave } from './wallet.js';
import {
    renderExistingCalendarConnections,
    populatePrimaryCalendarOptions
} from './calendars.js';
import { BOT } from '/js/shared/app-config.js';

export function setDebugStatus(text) {
    els.debugStatus.textContent = text || '';
}

export function populateForm(data) {
    state.currentExpert = data;

    els.displayName.value = data.display_name || '';
    els.telegramBio.value = data.telegram_bio || '';
    els.usernameReadonly.value = data.username ? `@${data.username}` : '';
    els.publicLinkReadonly.value = data.public_slug
        ? `https://t.me/${BOT}?startapp=${encodeURIComponent(data.public_slug)}`
        : '';
    els.timezoneReadonly.value = data.timezone || '';
    els.hourlyRate.value = data.hourly_rate || '';
    els.currency.value = data.currency || '';
    els.workStart.value = data.work_start_time || '';
    els.workEnd.value = data.work_end_time || '';
    els.minimumNotice.value = data.minimum_notice_minutes ?? 60;
    els.maxDaysAhead.value = data.max_days_ahead ?? 30;
    els.bufferBefore.value = data.buffer_before_minutes ?? 0;
    els.bufferAfter.value = data.buffer_after_minutes ?? 0;

    setChipGroupValues(els.workingDaysGroup, data.working_days || []);
    setChipGroupValues(els.durationsGroup, data.allowed_session_durations || []);

    els.isActive.checked = !!data.is_active;
    els.isBookable.checked = !!data.is_bookable;

    state.existingCalendarConnections = Array.isArray(data.calendar_connections)
        ? data.calendar_connections
        : [];

    refreshWalletUi();
    renderExistingCalendarConnections();
    populatePrimaryCalendarOptions(
        data.calendar_connections || [],
        data.primary_calendar_connection_id
    );

    if (state.currentTelegramUser?.id && Number(state.currentTelegramUser.id) !== Number(data.telegram_id)) {
        setDebugStatus('This Telegram account does not own this expert profile.');
        els.saveProfileBtn.disabled = true;
    } else {
        setDebugStatus('');
        els.saveProfileBtn.disabled = !state.currentTelegramUser?.id;
    }
}

if (els.copyPublicLinkBtn) {
    els.copyPublicLinkBtn.onclick = async () => {
        const value = els.publicLinkReadonly?.value || '';
        if (!value) {
            setDebugStatus('Public link is empty.');
            return;
        }

        try {
            await navigator.clipboard.writeText(value);
            setDebugStatus('Public link copied.');
        } catch (error) {
            console.error(error);
            setDebugStatus('Copy failed.');
        }
    };
}

export function buildPayload() {
    return {
        telegram_id: state.currentTelegramUser.id,
        display_name: els.displayName.value.trim(),
        telegram_bio: els.telegramBio.value.trim() || null,
        ton_wallet_address: getWalletAddressForSave(),
        hourly_rate: els.hourlyRate.value.trim(),
        currency: els.currency.value.trim(),
        working_days: getChipGroupValues(els.workingDaysGroup, false),
        work_start_time: els.workStart.value,
        work_end_time: els.workEnd.value,
        allowed_session_durations: getChipGroupValues(els.durationsGroup, true),
        minimum_notice_minutes: Number(els.minimumNotice.value || 0),
        buffer_before_minutes: Number(els.bufferBefore.value || 0),
        buffer_after_minutes: Number(els.bufferAfter.value || 0),
        max_days_ahead: Number(els.maxDaysAhead.value || 30),
        is_active: els.isActive.checked,
        is_bookable: els.isBookable.checked,
        primary_calendar_connection_id: els.primaryCalendar?.value
            ? Number(els.primaryCalendar.value)
            : null,
        attach_google_session_ids: state.pendingGoogleSessionIds,
    };
}