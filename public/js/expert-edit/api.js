import { els } from './dom.js';
import { state } from './state.js';
import { getSlugFromPath } from './utils.js';
import { buildPayload, populateForm, setDebugStatus } from './form.js';

export async function loadExpertData() {
    const slug = getSlugFromPath();
    if (!slug) {
        setDebugStatus('Missing slug in URL.');
        return;
    }

    try {
        setDebugStatus('Loading profile...');
        const response = await fetch(`/api/experts/${encodeURIComponent(slug)}/edit`);

        if (!response.ok) {
            const text = await response.text().catch(() => '');
            throw new Error(`${response.status} ${text}`);
        }

        const data = await response.json();
        state.pendingGoogleSessionIds = [];
        populateForm(data);
        setDebugStatus('');
    } catch (error) {
        console.error(error);
        setDebugStatus(`Failed to load profile: ${error.message}`);
    }
}

export async function saveExpertData() {
    const slug = getSlugFromPath();
    if (!slug) {
        setDebugStatus('Missing slug in URL.');
        return;
    }

    if (!state.currentTelegramUser?.id) {
        setDebugStatus('Connect Telegram first.');
        return;
    }

    try {
        setDebugStatus('Saving...');
        els.saveProfileBtn.disabled = true;

        const response = await fetch(`/api/experts/${encodeURIComponent(slug)}/edit`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(buildPayload())
        });

        if (!response.ok) {
            const text = await response.text().catch(() => '');
            throw new Error(`${response.status} ${text}`);
        }

        const data = await response.json();
        populateForm(data);
        setDebugStatus('Profile saved.');
    } catch (error) {
        console.error(error);
        setDebugStatus(`Save failed: ${error.message}`);
    } finally {
        if (state.currentTelegramUser?.id) {
            els.saveProfileBtn.disabled = false;
        }
    }
}

export async function previewDeleteExpertProfile() {
    const slug = getSlugFromPath();

    if (!slug) {
        throw new Error('Missing slug in URL.');
    }

    if (!state.currentTelegramUser?.id) {
        throw new Error('Connect Telegram first.');
    }

    const response = await fetch(`/api/experts/${encodeURIComponent(slug)}/delete-preview`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            telegram_id: state.currentTelegramUser.id
        })
    });

    const data = await response.json().catch(() => ({}));

    if (!response.ok) {
        throw new Error(data.error || `Delete preview failed: ${response.status}`);
    }

    return data;
}

export async function deleteExpertProfile(confirmPaidFutureBookings = false) {
    const slug = getSlugFromPath();

    if (!slug) {
        throw new Error('Missing slug in URL.');
    }

    if (!state.currentTelegramUser?.id) {
        throw new Error('Connect Telegram first.');
    }

    const response = await fetch(`/api/experts/${encodeURIComponent(slug)}`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            telegram_id: state.currentTelegramUser.id,
            confirm_paid_future_bookings: confirmPaidFutureBookings
        })
    });

    const data = await response.json().catch(() => ({}));

    if (!response.ok) {
        throw new Error(data.error || `Delete failed: ${response.status}`);
    }

    return data;
}