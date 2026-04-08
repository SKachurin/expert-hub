import { PUBLIC_ORIGIN } from './config.js';
import { els } from './dom.js';
import { state } from './state.js';

export function getSavedWalletAddress() {
    return state.currentExpert?.ton_wallet_address || '';
}

export function getPendingWalletAddress() {
    return state.pendingWallet?.account?.address || '';
}

export function getWalletAddressForSave() {
    return getPendingWalletAddress() || getSavedWalletAddress();
}

export function refreshWalletUi() {
    els.walletReadonly.value = getSavedWalletAddress();
    if (els.newWalletReadonly) {
        els.newWalletReadonly.value = getPendingWalletAddress();
    }
}

export function initTonConnectForEdit() {
    if (!window.TON_CONNECT_UI) {
        console.error('TON_CONNECT_UI is missing');
        return;
    }

    state.tonUi = new TON_CONNECT_UI.TonConnectUI({
        manifestUrl: `${PUBLIC_ORIGIN}/tonconnect-manifest.json`,
        buttonRootId: 'ton-connect'
    });

    state.tonUi.onStatusChange((wallet) => {
        state.pendingWallet = wallet || null;
        refreshWalletUi();
    });
}