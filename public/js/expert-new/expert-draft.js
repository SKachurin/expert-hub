import { EXPERT_DRAFT_KEY } from '../shared/app-config.js';

export const expertDraft = {
    display_name: '',
    telegram_bio: '',
    timezone: '',
    hourly_rate: '1.00',
    currency: 'USD',
    working_days: ['mon', 'tue', 'wed', 'thu', 'fri'],
    work_start_time: '10:00',
    work_end_time: '17:00',
    allowed_session_durations: [30, 60]
};

export function saveExpertDraft() {
    localStorage.setItem(EXPERT_DRAFT_KEY, JSON.stringify(expertDraft));
}

export function loadExpertDraft() {
    try {
        const raw = localStorage.getItem(EXPERT_DRAFT_KEY);
        if (!raw) return;

        const parsed = JSON.parse(raw);
        if (!parsed || typeof parsed !== 'object') return;

        expertDraft.display_name = parsed.display_name || '';
        expertDraft.telegram_bio = parsed.telegram_bio || '';
        expertDraft.timezone = parsed.timezone || '';
        expertDraft.hourly_rate = parsed.hourly_rate || '1.00';
        expertDraft.currency = parsed.currency || 'USD';
        expertDraft.working_days = Array.isArray(parsed.working_days)
            ? parsed.working_days
            : ['mon', 'tue', 'wed', 'thu', 'fri'];
        expertDraft.work_start_time = parsed.work_start_time || '10:00';
        expertDraft.work_end_time = parsed.work_end_time || '17:00';
        expertDraft.allowed_session_durations = Array.isArray(parsed.allowed_session_durations)
            ? parsed.allowed_session_durations
            : [30, 60];
    } catch (e) {
        console.error('Failed to load expert draft', e);
    }
}