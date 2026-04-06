import {
    CALENDAR_DRAFT_KEY,
    MAX_CALENDAR_BLOCKS
} from '../shared/app-config.js';

export const calendarDraft = {
    blocks: [
        {
            id: crypto.randomUUID(),
            provider: '',
            connected: false,
            google_session_id: '',
            google_account_email: '',
            google_calendars: [],
            selected_calendar_ids: []
        }
    ]
};

export function saveCalendarDraft() {
    localStorage.setItem(CALENDAR_DRAFT_KEY, JSON.stringify(calendarDraft));
}

export function loadCalendarDraft() {
    try {
        const raw = localStorage.getItem(CALENDAR_DRAFT_KEY);
        if (!raw) return;

        const parsed = JSON.parse(raw);

        if (parsed && Array.isArray(parsed.blocks) && parsed.blocks.length > 0) {
            calendarDraft.blocks = parsed.blocks.map((block) => ({
                id: block.id || crypto.randomUUID(),
                provider: block.provider || '',
                connected: !!block.connected,
                google_session_id: block.google_session_id || '',
                google_account_email: block.google_account_email || '',
                google_calendars: Array.isArray(block.google_calendars) ? block.google_calendars : [],
                selected_calendar_ids: Array.isArray(block.selected_calendar_ids) ? block.selected_calendar_ids : []
            }));
        }
    } catch (e) {
        console.error('Failed to load calendar draft', e);
    }
}

export function ensureCalendarDraftBlock(number) {
    const index = number - 1;

    if (!calendarDraft.blocks[index]) {
        calendarDraft.blocks[index] = {
            id: crypto.randomUUID(),
            provider: '',
            connected: false,
            google_session_id: '',
            google_account_email: '',
            google_calendars: [],
            selected_calendar_ids: []
        };
    }
}

export function canAddAnotherCalendarBlock() {
    return calendarDraft.blocks.length < MAX_CALENDAR_BLOCKS;
}