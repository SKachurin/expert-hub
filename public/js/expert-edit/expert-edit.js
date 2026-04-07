const APP_HOST = window.location.hostname;
const PUBLIC_ORIGIN = window.location.origin;
const IS_DEV = APP_HOST === 'dev.experthub.bar';

const BOT = IS_DEV ? 'expert_hub_bot' : 'experthub_bbot';
const TG_APP_LINK = `https://t.me/${BOT}?startapp=expert_new`;

const TELEGRAM_USER_KEY = IS_DEV
    ? 'Dev_expertHubTelegramUserV1'
    : 'expertHubTelegramUserV1';

const TELEGRAM_USER_TTL_MS = 6 * 60 * 60 * 1000;

const debugStatusEl = document.getElementById('debug-status');
const profileCardEl = document.getElementById('profile-card');

const displayNameEl = document.getElementById('display-name');
const telegramBioEl = document.getElementById('telegram-bio');
const usernameReadonlyEl = document.getElementById('username-readonly');
const publicSlugReadonlyEl = document.getElementById('public-slug-readonly');
const timezoneReadonlyEl = document.getElementById('timezone-readonly');
const walletReadonlyEl = document.getElementById('wallet-readonly');

const hourlyRateEl = document.getElementById('hourly-rate');
const currencyEl = document.getElementById('currency');
const workStartEl = document.getElementById('work-start');
const workEndEl = document.getElementById('work-end');
const minimumNoticeEl = document.getElementById('minimum-notice');
const maxDaysAheadEl = document.getElementById('max-days-ahead');
const bufferBeforeEl = document.getElementById('buffer-before');
const bufferAfterEl = document.getElementById('buffer-after');

const workingDaysGroupEl = document.getElementById('working-days-group');
const durationsGroupEl = document.getElementById('durations-group');
const primaryCalendarEl = document.getElementById('primary-calendar');

const isActiveEl = document.getElementById('is-active');
const isBookableEl = document.getElementById('is-bookable');

const saveProfileBtnEl = document.getElementById('save-profile-btn');
const newWalletReadonlyEl = document.getElementById('new-wallet-readonly');
const existingCalendarListEl = document.getElementById('existing-calendar-list');
const connectGoogleCalendarBtnEl = document.getElementById('connect-google-calendar-btn');

let currentTelegramUser = null;
let currentExpert = null;
let pendingWallet = null;
let tonUi = null;
let existingCalendarConnections = [];
let pendingGoogleSessionIds = [];


function getSavedWalletAddress() {
    return currentExpert?.ton_wallet_address || '';
}

function getPendingWalletAddress() {
    return pendingWallet?.account?.address || '';
}

function getWalletAddressForSave() {
    return getPendingWalletAddress() || getSavedWalletAddress();
}
function refreshWalletUi() {
    walletReadonlyEl.value = getSavedWalletAddress();
    if (newWalletReadonlyEl) {
        newWalletReadonlyEl.value = getPendingWalletAddress();
    }
}
function initTonConnectForEdit() {
    if (!window.TON_CONNECT_UI) {
        console.error('TON_CONNECT_UI is missing');
        return;
    }

    tonUi = new TON_CONNECT_UI.TonConnectUI({
        manifestUrl: `${PUBLIC_ORIGIN}/tonconnect-manifest.json`,
        buttonRootId: 'ton-connect'
    });

    tonUi.onStatusChange((wallet) => {
        pendingWallet = wallet || null;
        refreshWalletUi();
    });
}
function setDebugStatus(text) {
    debugStatusEl.textContent = text || '';
}

function escapeHtml(value) {
    return String(value ?? '')
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#039;');
}

function telegramIconSvg() {
    return `
        <svg viewBox="0 0 240 240" xmlns="http://www.w3.org/2000/svg">
            <path d="M179.6 71.3L160.4 168c-1.4 7.3-5.6 9.1-11.3 5.7l-31.3-23.1-15.1 14.6c-1.7 1.7-3.1 3.1-6.4 3.1l2.3-32.4 58.9-53.2c2.6-2.3-0.6-3.6-4-1.3l-72.8 45.9-31.4-9.8c-6.8-2.1-7-6.7 1.4-10l122.7-47.3c5.7-2.1 10.6 1.3 8.8 9.1z" fill="#ffffff"/>
        </svg>
    `;
}

function initialsFromUser(user) {
    const a = (user?.first_name || '').trim().charAt(0);
    const b = (user?.last_name || '').trim().charAt(0);
    return (a + b).trim() || 'T';
}

function fullName(user) {
    return [user?.first_name || '', user?.last_name || ''].join(' ').trim() || user?.username || 'Telegram user';
}

function getSlugFromPath() {
    const parts = window.location.pathname.split('/').filter(Boolean);
    if (parts.length >= 3 && parts[0] === 'e' && parts[2] === 'edit') {
        return parts[1];
    }
    return '';
}

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

function getMiniAppTelegramUser() {
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

function resolveTelegramUser() {
    const miniUser = getMiniAppTelegramUser();
    if (miniUser) {
        return saveTelegramUser(miniUser);
    }
    return getStoredTelegramUser();
}

function mountTelegramLoginWidget() {
    const hostEl = document.getElementById('telegram-login-slot');
    if (!hostEl) return;

    hostEl.innerHTML = '';

    const iframe = document.createElement('iframe');
    iframe.src = `https://oauth.telegram.org/embed/${BOT}?origin=${encodeURIComponent(PUBLIC_ORIGIN)}&request_access=write&embed=1`;
    iframe.width = 360;
    iframe.height = 54;
    iframe.style.border = '0';
    iframe.style.overflow = 'hidden';
    iframe.setAttribute('scrolling', 'no');

    hostEl.appendChild(iframe);
}

function renderProfileCard(user) {
    if (!user) {
        profileCardEl.innerHTML = `
            <div class="profile-top">
                <div class="avatar avatar-placeholder">${escapeHtml(initialsFromUser(null))}</div>
                <div class="profile-copy">
                    <div id="telegram-login-slot" class="telegram-login-slot"></div>
                    <div class="profile-sub">Telegram account not connected</div>
                    <a class="telegram-open-link" href="${TG_APP_LINK}" target="_blank" rel="noopener noreferrer">
                        Open in Telegram
                    </a>
                </div>
            </div>
        `;
        mountTelegramLoginWidget();
        saveProfileBtnEl.disabled = true;
        return;
    }

    const photo = user.photo_url
        ? `<img class="avatar" src="${escapeHtml(user.photo_url)}" alt="">`
        : `<div class="avatar avatar-placeholder">${escapeHtml(initialsFromUser(user))}</div>`;

    profileCardEl.innerHTML = `
        <div class="profile-top">
            ${photo}
            <div class="profile-copy">
                <div class="identity-pill">
                    <span class="identity-icon">${telegramIconSvg()}</span>
                    <span>${escapeHtml(fullName(user))}</span>
                </div>
                <div class="profile-sub">Telegram connected</div>
            </div>
        </div>
    `;
}

function setChipGroupValues(groupEl, values) {
    const set = new Set(values || []);

    groupEl.querySelectorAll('.check-chip').forEach((chip) => {
        const input = chip.querySelector('input');
        const isActive = set.has(input.value) || set.has(Number(input.value));
        input.checked = isActive;
        chip.classList.toggle('active', isActive);
    });
}

function getChipGroupValues(groupEl, asNumbers = false) {
    return Array.from(groupEl.querySelectorAll('input:checked')).map((input) => {
        return asNumbers ? Number(input.value) : input.value;
    });
}

function bindChipGroup(groupEl) {
    groupEl.querySelectorAll('.check-chip').forEach((chip) => {
        chip.addEventListener('click', () => {
            const input = chip.querySelector('input');
            input.checked = !input.checked;
            chip.classList.toggle('active', input.checked);
        });
    });
}

function populateForm(data) {
    currentExpert = data;

    displayNameEl.value = data.display_name || '';
    telegramBioEl.value = data.telegram_bio || '';
    usernameReadonlyEl.value = data.username ? `@${data.username}` : '';
    publicSlugReadonlyEl.value = data.public_slug || '';
    timezoneReadonlyEl.value = data.timezone || '';
    hourlyRateEl.value = data.hourly_rate || '';
    currencyEl.value = data.currency || '';
    workStartEl.value = data.work_start_time || '';
    workEndEl.value = data.work_end_time || '';
    minimumNoticeEl.value = data.minimum_notice_minutes ?? 60;
    maxDaysAheadEl.value = data.max_days_ahead ?? 30;
    bufferBeforeEl.value = data.buffer_before_minutes ?? 0;
    bufferAfterEl.value = data.buffer_after_minutes ?? 0;

    setChipGroupValues(workingDaysGroupEl, data.working_days || []);
    setChipGroupValues(durationsGroupEl, data.allowed_session_durations || []);

    isActiveEl.checked = !!data.is_active;
    isBookableEl.checked = !!data.is_bookable;

    existingCalendarConnections = Array.isArray(data.calendar_connections)
        ? data.calendar_connections
        : [];

    refreshWalletUi();
    renderExistingCalendarConnections();
    populatePrimaryCalendarOptions(
        data.calendar_connections || [],
        data.primary_calendar_connection_id
    );

    if (currentTelegramUser?.id && Number(currentTelegramUser.id) !== Number(data.telegram_id)) {
        setDebugStatus('This Telegram account does not own this expert profile.');
        saveProfileBtnEl.disabled = true;
    } else {
        setDebugStatus('');
        saveProfileBtnEl.disabled = !currentTelegramUser?.id;
    }
}

function buildPayload() {
    return {
        telegram_id: currentTelegramUser.id,
        display_name: displayNameEl.value.trim(),
        telegram_bio: telegramBioEl.value.trim() || null,
        ton_wallet_address: getWalletAddressForSave(),
        hourly_rate: hourlyRateEl.value.trim(),
        currency: currencyEl.value.trim(),
        working_days: getChipGroupValues(workingDaysGroupEl, false),
        work_start_time: workStartEl.value,
        work_end_time: workEndEl.value,
        allowed_session_durations: getChipGroupValues(durationsGroupEl, true),
        minimum_notice_minutes: Number(minimumNoticeEl.value || 0),
        buffer_before_minutes: Number(bufferBeforeEl.value || 0),
        buffer_after_minutes: Number(bufferAfterEl.value || 0),
        max_days_ahead: Number(maxDaysAheadEl.value || 30),
        is_active: isActiveEl.checked,
        is_bookable: isBookableEl.checked,
        primary_calendar_connection_id: primaryCalendarEl && primaryCalendarEl.value
            ? Number(primaryCalendarEl.value)
            : null,
        attach_google_session_ids: pendingGoogleSessionIds,
    };
}

async function loadExpertData() {
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
        pendingGoogleSessionIds = [];
        populateForm(data);
        setDebugStatus('');
    } catch (error) {
        console.error(error);
        setDebugStatus(`Failed to load profile: ${error.message}`);
    }
}

async function saveExpertData() {
    const slug = getSlugFromPath();
    if (!slug) {
        setDebugStatus('Missing slug in URL.');
        return;
    }

    if (!currentTelegramUser?.id) {
        setDebugStatus('Connect Telegram first.');
        return;
    }

    try {
        setDebugStatus('Saving...');
        saveProfileBtnEl.disabled = true;

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
        if (currentTelegramUser?.id) {
            saveProfileBtnEl.disabled = false;
        }
    }
}

function renderExistingCalendarConnections() {
    if (!existingCalendarListEl) return;

    existingCalendarListEl.innerHTML = '';

    if (!Array.isArray(existingCalendarConnections) || !existingCalendarConnections.length) {
        existingCalendarListEl.innerHTML = '<div class="section-note">No connected calendars yet.</div>';
        return;
    }

    existingCalendarConnections.forEach((item) => {
        const card = document.createElement('div');
        card.className = 'existing-calendar-card';
        card.innerHTML = `
            <button
                type="button"
                class="existing-calendar-delete"
                data-connection-id="${item.id}"
                aria-label="Delete calendar"
            >×</button>

            <div><strong>${escapeHtml(item.connection_label || item.provider || 'Calendar')}</strong></div>
            ${item.selected_calendar_name ? `<div>${escapeHtml(item.selected_calendar_name)}</div>` : ''}
            ${item.selected_calendar_timezone ? `<div>${escapeHtml(item.selected_calendar_timezone)}</div>` : ''}
        `;
        existingCalendarListEl.appendChild(card);
    });

    existingCalendarListEl.querySelectorAll('.existing-calendar-delete').forEach((btn) => {
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

                existingCalendarConnections = existingCalendarConnections.filter(
                    (item) => String(item.id) !== String(connectionId)
                );

                renderExistingCalendarConnections();
                populatePrimaryCalendarOptions(existingCalendarConnections, null);
                setDebugStatus('Calendar deleted.');
            } catch (error) {
                console.error(error);
                btn.disabled = false;
                setDebugStatus(`Delete failed: ${error.message}`);
            }
        });
    });
}

function connectAnotherGoogleCalendar() {
    if (!currentTelegramUser?.id) {
        setDebugStatus('Connect Telegram first.');
        return;
    }

    window.location.href =
        `/oauth/google/start?telegram_id=${encodeURIComponent(currentTelegramUser.id)}&return_to=expert_edit&slug=${encodeURIComponent(getSlugFromPath())}`;
}

function consumeGoogleSessionFromUrl() {
    const url = new URL(window.location.href);
    const sessionId = url.searchParams.get('google_session');

    if (!sessionId) {
        return null;
    }

    url.searchParams.delete('google_session');
    window.history.replaceState({}, '', url.toString());

    return sessionId;
}

function addPendingGoogleSession(sessionId) {
    if (!sessionId) return;
    if (!pendingGoogleSessionIds.includes(sessionId)) {
        pendingGoogleSessionIds.push(sessionId);
    }
}

function populatePrimaryCalendarOptions(items, selectedId) {
    if (!primaryCalendarEl) return;

    primaryCalendarEl.innerHTML = '';

    if (!Array.isArray(items) || !items.length) {
        const option = document.createElement('option');
        option.value = '';
        option.textContent = 'No connected calendars';
        primaryCalendarEl.appendChild(option);
        return;
    }

    items.forEach((item) => {
        const option = document.createElement('option');
        option.value = String(item.id);
        option.textContent = item.connection_label || item.provider || `Calendar ${item.id}`;
        option.selected = Number(selectedId) === Number(item.id);
        primaryCalendarEl.appendChild(option);
    });
}

window.addEventListener('message', async (e) => {
    if (e.origin !== 'https://oauth.telegram.org') return;

    let payload = e.data;
    if (typeof payload === 'string') {
        try {
            payload = JSON.parse(payload);
        } catch {
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
        currentTelegramUser = saveTelegramUser(user);
        renderProfileCard(currentTelegramUser);
        await loadExpertData();
    }
});

document.addEventListener('DOMContentLoaded', async () => {
    bindChipGroup(workingDaysGroupEl);
    bindChipGroup(durationsGroupEl);

    const tg = window.Telegram?.WebApp;
    if (tg) {
        try {
            tg.ready();
            tg.expand();
        } catch (e) {
            console.error('Telegram WebApp init error on edit page:', e);
        }
    }

    currentTelegramUser = resolveTelegramUser();
    renderProfileCard(currentTelegramUser);

    await loadExpertData();
    initTonConnectForEdit();

    const returnedGoogleSessionId = consumeGoogleSessionFromUrl();
    if (returnedGoogleSessionId) {
        addPendingGoogleSession(returnedGoogleSessionId);
        setDebugStatus('Google calendar connected. Save profile to attach it.');
    }
    if (connectGoogleCalendarBtnEl) {
        connectGoogleCalendarBtnEl.addEventListener('click', connectAnotherGoogleCalendar);
    }

    saveProfileBtnEl.addEventListener('click', saveExpertData);
});