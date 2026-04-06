import { els } from './dom.js';
import { expertDraft } from './expert-draft.js';
import { calendarDraft } from './calendar-draft.js';
import { TG_APP_LINK } from '../shared/app-config.js';
import {
    escapeHtml,
    fullName,
    humanChain,
    initialsFromUser,
    shortAddress
} from '../shared/dom-utils.js';

export function setDebugStatus(text) {
    els.debugStatus.textContent = text || '';
}

export function setStepState(el, text, ok) {
    if (!el) return;
    el.textContent = text;
    el.classList.toggle('ok', !!ok);
}

export function setBadgeState(el, connected, connectedText, disconnectedText) {
    if (!el) return;
    el.textContent = connected ? connectedText : disconnectedText;
    el.classList.toggle('connected', connected);
    el.classList.toggle('disconnected', !connected);
}

export function telegramIconSvg() {
    return `
        <svg viewBox="0 0 240 240" xmlns="http://www.w3.org/2000/svg">
            <path d="M179.6 71.3L160.4 168c-1.4 7.3-5.6 9.1-11.3 5.7l-31.3-23.1-15.1 14.6c-1.7 1.7-3.1 3.1-6.4 3.1l2.3-32.4 58.9-53.2c2.6-2.3-0.6-3.6-4-1.3l-72.8 45.9-31.4-9.8c-6.8-2.1-7-6.7 1.4-10l122.7-47.3c5.7-2.1 10.6 1.3 8.8 9.1z" fill="#ffffff"/>
        </svg>
    `;
}

export function mountTelegramLoginWidget(bot) {
    const hostEl = document.getElementById('telegram-login-slot');
    if (!hostEl) return;

    hostEl.innerHTML = '';

    const iframe = document.createElement('iframe');
    iframe.src = `https://oauth.telegram.org/embed/${bot}?origin=${encodeURIComponent(window.location.origin)}&request_access=write&embed=1`;
    iframe.width = 360;
    iframe.height = 54;
    iframe.style.border = '0';
    iframe.style.overflow = 'hidden';
    iframe.setAttribute('scrolling', 'no');

    hostEl.appendChild(iframe);
}

export function renderProfileCard(user, bot) {
    if (!user) {
        els.profileCard.innerHTML = `
            <div class="profile-top">
                <div class="avatar avatar-placeholder">${escapeHtml(initialsFromUser(null))}</div>
                <div class="profile-copy">
                    <div id="telegram-login-slot" class="telegram-login-slot"></div>
                    <div class="profile-sub">
                        <span style="width:8px;height:8px;border-radius:50%;background:var(--danger);display:inline-block;"></span>
                        Telegram account not connected
                    </div>
                    <a class="telegram-open-link" href="${TG_APP_LINK}" target="_blank" rel="noopener noreferrer">
                        Open in Telegram
                    </a>
                </div>
            </div>
        `;

        mountTelegramLoginWidget(bot);
        setStepState(els.stepTelegram, 'Pending', false);
        return;
    }

    const photo = user.photo_url
        ? `<img class="avatar" src="${escapeHtml(user.photo_url)}" alt="">`
        : `<div class="avatar avatar-placeholder">${escapeHtml(initialsFromUser(user))}</div>`;

    els.profileCard.innerHTML = `
        <div class="profile-top">
            ${photo}
            <div class="profile-copy">
                <div class="identity-pill">
                    <span class="identity-icon">${telegramIconSvg()}</span>
                    <span>${escapeHtml(fullName(user))}</span>
                </div>
                <div class="profile-sub">
                    <span class="dot"></span>
                    Telegram connected
                </div>
            </div>
        </div>
    `;

    setStepState(els.stepTelegram, 'Connected', true);

    if (!expertDraft.display_name.trim()) {
        expertDraft.display_name = fullName(user);
    }

    if (!els.displayName.value.trim()) {
        els.displayName.value = expertDraft.display_name;
    }
}

export function updateWalletUi(wallet, setCurrentWallet, updateRegisterButtonState) {
    setCurrentWallet(wallet);

    if (!wallet) {
        setBadgeState(els.walletBadge, false, 'Connected', 'Not connected');
        els.walletShort.textContent = 'No wallet';
        els.walletFull.textContent = 'Connect your TON wallet to continue setup.';
        els.walletChain.classList.add('hidden');
        els.walletChain.textContent = '';
        setStepState(els.stepWallet, 'Pending', false);
        updateRegisterButtonState();
        return;
    }

    const addr = wallet.account.address;
    const chain = wallet.account.chain;

    setBadgeState(els.walletBadge, true, 'Connected', 'Not connected');
    els.walletShort.textContent = shortAddress(addr);
    els.walletFull.textContent = addr;
    els.walletChain.textContent = humanChain(chain);
    els.walletChain.classList.remove('hidden');
    setStepState(els.stepWallet, 'Connected', true);
    updateRegisterButtonState();
}

export function syncExpertDraftToForm() {
    els.displayName.value = expertDraft.display_name;
    els.telegramBio.value = expertDraft.telegram_bio;
    els.timezone.value = expertDraft.timezone;
    els.hourlyRate.value = expertDraft.hourly_rate;
    els.currency.value = expertDraft.currency;
    els.workStart.value = expertDraft.work_start_time;
    els.workEnd.value = expertDraft.work_end_time;

    els.workingDaysGroup.querySelectorAll('.check-chip').forEach((chip) => {
        const input = chip.querySelector('input');
        const checked = expertDraft.working_days.includes(input.value);
        input.checked = checked;
        chip.classList.toggle('active', checked);
    });

    els.durationsGroup.querySelectorAll('.check-chip').forEach((chip) => {
        const input = chip.querySelector('input');
        const checked = expertDraft.allowed_session_durations.includes(Number(input.value));
        input.checked = checked;
        chip.classList.toggle('active', checked);
    });
}

export function validateProfileDraft(updateRegisterButtonState) {
    const ok =
        els.displayName.value.trim() !== '' &&
        els.timezone.value.trim() !== '' &&
        els.hourlyRate.value.trim() !== '' &&
        Number(els.hourlyRate.value) >= 1 &&
        els.currency.value.trim() !== '' &&
        els.workStart.value.trim() !== '' &&
        els.workEnd.value.trim() !== '' &&
        expertDraft.working_days.length > 0 &&
        expertDraft.allowed_session_durations.length > 0;

    setBadgeState(els.profileBadge, ok, 'Filled', 'Not filled');
    setStepState(els.stepProfile, ok ? 'Filled' : 'Pending', ok);
    updateRegisterButtonState();

    return ok;
}

export function updateRegisterButtonState(currentTelegramUser, currentWallet) {
    const telegramReady = !!currentTelegramUser?.id;
    const walletReady = !!currentWallet?.account?.address;

    const profileReady =
        els.displayName.value.trim() !== '' &&
        els.timezone.value.trim() !== '' &&
        els.hourlyRate.value.trim() !== '' &&
        Number(els.hourlyRate.value) >= 1 &&
        els.currency.value.trim() !== '' &&
        els.workStart.value.trim() !== '' &&
        els.workEnd.value.trim() !== '' &&
        expertDraft.working_days.length > 0 &&
        expertDraft.allowed_session_durations.length > 0;

    const connectedCalendars = calendarDraft.blocks.filter(block => block.connected);
    const calendarReady = connectedCalendars.length >= 1;

    const ready = telegramReady && walletReady && profileReady && calendarReady;

    els.registerExpertBtn.classList.toggle('btn-disabled', !ready);
    els.registerExpertBtn.disabled = !ready;
}