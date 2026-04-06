import { expertDraft } from './expert-draft.js';
import { calendarDraft } from './calendar-draft.js';

export function buildExpertPayload(currentTelegramUser, currentWallet) {
    return {
        telegram_id: currentTelegramUser.id,
        first_name: currentTelegramUser.first_name || '',
        last_name: currentTelegramUser.last_name || '',
        username: currentTelegramUser.username || '',
        photo_url: currentTelegramUser.photo_url || null,
        ton_wallet_address: currentWallet.account.address,
        timezone: expertDraft.timezone,
        display_name: expertDraft.display_name,
        telegram_bio: expertDraft.telegram_bio,
        hourly_rate: expertDraft.hourly_rate,
        currency: expertDraft.currency,
        working_days: expertDraft.working_days,
        work_start_time: expertDraft.work_start_time,
        work_end_time: expertDraft.work_end_time,
        allowed_session_durations: expertDraft.allowed_session_durations,
        calendar_connections: calendarDraft.blocks
            .filter((block) => block.connected)
            .map((block) => ({
                provider: block.provider,
                google_session_id: block.provider === 'google' ? block.google_session_id : null
            }))
    };
}

export async function registerExpert(currentTelegramUser, currentWallet) {
    const payload = buildExpertPayload(currentTelegramUser, currentWallet);

    const response = await fetch('/expert-setup/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
    });

    if (!response.ok) {
        const text = await response.text().catch(() => '');
        throw new Error(`/expert-setup/register failed: ${response.status} ${text}`);
    }

    return response.json();
}