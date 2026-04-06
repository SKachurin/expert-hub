export function escapeHtml(value) {
    return String(value ?? '')
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#039;');
}

export function shortAddress(address) {
    if (!address || address.length < 10) return address || '';
    return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

export function humanChain(chain) {
    if (chain === -239) return 'Mainnet';
    if (chain === -3) return 'Testnet';
    return `Chain ${chain}`;
}

export function fullName(user) {
    return [user?.first_name || '', user?.last_name || ''].join(' ').trim()
        || user?.username
        || 'Telegram user';
}

export function initialsFromUser(user) {
    const a = (user?.first_name || '').trim().charAt(0);
    const b = (user?.last_name || '').trim().charAt(0);
    return (a + b).trim() || 'T';
}