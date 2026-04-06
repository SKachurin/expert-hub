export const APP_HOST = window.location.hostname;
export const PUBLIC_ORIGIN = window.location.origin;
export const IS_DEV = APP_HOST === 'dev.experthub.bar';

export const BOT = IS_DEV ? 'expert_hub_bot' : 'experthub_bbot';
export const TG_APP_LINK = `https://t.me/${BOT}?startapp=expert_new`;
export const MANIFEST_URL = `${PUBLIC_ORIGIN}/tonconnect-manifest.json`;

export const TELEGRAM_USER_KEY = IS_DEV
    ? 'Dev_expertHubTelegramUserV1'
    : 'expertHubTelegramUserV1';

export const TELEGRAM_USER_TTL_MS = 6 * 60 * 60 * 1000;

export const EXPERT_DRAFT_KEY = IS_DEV
    ? 'Dev_expertHubExpertDraftV1'
    : 'expertHubExpertDraftV1';

export const CALENDAR_DRAFT_KEY = IS_DEV
    ? 'Dev_expertHubCalendarDraftV1'
    : 'expertHubCalendarDraftV1';

export const MAX_CALENDAR_BLOCKS = 5;