import { BOT } from '/js/shared/app-config.js';

const debugStatusEl = document.getElementById('debug-status');
const publicUrlEl = document.getElementById('public-url');
const editUrlEl = document.getElementById('edit-url');
const openPublicBtnEl = document.getElementById('open-public-btn');
const openEditBtnEl = document.getElementById('open-edit-btn');
const copyPublicBtnEl = document.getElementById('copy-public-btn');

function setDebugStatus(text) {
    debugStatusEl.textContent = text || '';
}

function getSlugFromUrl() {
    const url = new URL(window.location.href);
    return (url.searchParams.get('slug') || '').trim();
}

async function copyText(value, successText) {
    try {
        await navigator.clipboard.writeText(value);
        setDebugStatus(successText);
    } catch (err) {
        console.error(err);
        setDebugStatus('Copy failed.');
    }
}
function buildTelegramMiniAppPublicLink(slug) {
    return `https://t.me/${BOT}?startapp=${encodeURIComponent(slug)}`;
}

function initCreatedPage() {
    const slug = getSlugFromUrl();

    if (!slug) {
        setDebugStatus('Missing slug in URL.');
        publicUrlEl.textContent = 'Missing slug';
        editUrlEl.textContent = 'Missing slug';
        openPublicBtnEl.classList.add('hidden');
        openEditBtnEl.classList.add('hidden');
        copyPublicBtnEl.disabled = true;
        return;
    }

    const publicUrl = buildTelegramMiniAppPublicLink(slug);
    const editUrl = `${window.location.origin}/e/${slug}/edit`;

    publicUrlEl.textContent = publicUrl;
    editUrlEl.textContent = editUrl;

    openPublicBtnEl.href = publicUrl;
    openEditBtnEl.href = editUrl;

    copyPublicBtnEl.addEventListener('click', () => {
        copyText(publicUrl, 'Public link copied.');
    });
}

document.addEventListener('DOMContentLoaded', initCreatedPage);