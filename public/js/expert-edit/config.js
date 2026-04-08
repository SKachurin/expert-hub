export const APP_HOST = window.location.hostname;
export const PUBLIC_ORIGIN = window.location.origin;
export const IS_DEV = APP_HOST === 'dev.experthub.bar';

export const BOT = IS_DEV ? 'expert_hub_bot' : 'experthub_bbot';
export const TG_APP_LINK = `https://t.me/${BOT}?startapp=expert_new`;

export const TELEGRAM_USER_KEY = IS_DEV
    ? 'Dev_expertHubTelegramUserV1'
    : 'expertHubTelegramUserV1';

export const TELEGRAM_USER_TTL_MS = 6 * 60 * 60 * 1000;