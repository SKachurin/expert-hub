export function escapeHtml(value) {
    return String(value ?? '')
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#039;');
}

export function initialsFromUser(user) {
    const a = (user?.first_name || '').trim().charAt(0);
    const b = (user?.last_name || '').trim().charAt(0);
    return (a + b).trim() || 'T';
}

export function fullName(user) {
    return [user?.first_name || '', user?.last_name || ''].join(' ').trim()
        || user?.username
        || 'Telegram user';
}

export function getSlugFromPath() {
    const parts = window.location.pathname.split('/').filter(Boolean);
    if (parts.length >= 3 && parts[0] === 'e' && parts[2] === 'edit') {
        return parts[1];
    }
    return '';
}

export function telegramIconSvg() {
    return `
        <svg viewBox="0 0 240 240" xmlns="http://www.w3.org/2000/svg">
            <path d="M179.6 71.3L160.4 168c-1.4 7.3-5.6 9.1-11.3 5.7l-31.3-23.1-15.1 14.6c-1.7 1.7-3.1 3.1-6.4 3.1l2.3-32.4 58.9-53.2c2.6-2.3-0.6-3.6-4-1.3l-72.8 45.9-31.4-9.8c-6.8-2.1-7-6.7 1.4-10l122.7-47.3c5.7-2.1 10.6 1.3 8.8 9.1z" fill="#ffffff"/>
        </svg>
    `;
}