import { els } from './dom.js';
import { setDebugStatus } from './form.js';
import { previewDeleteExpertProfile, deleteExpertProfile } from './api.js';

function openModal(modal) {
    modal?.classList.remove('hidden');
}

function closeModal(modal) {
    modal?.classList.add('hidden');
}

function escapeHtml(value) {
    return String(value ?? '')
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#039;');
}

function renderPaidFutureBookings(bookings) {
    if (!els.deleteProfilePaidBookingsList) return;

    els.deleteProfilePaidBookingsList.innerHTML = bookings.map((booking) => {
        const customer = booking.customer_username
            ? `@${booking.customer_username}`
            : (booking.customer_display_name || `Telegram ID ${booking.customer_telegram_id}`);

        return `
            <div class="delete-booking-item">
                <div class="delete-booking-title">
                    Booking #${escapeHtml(booking.booking_id)} · ${escapeHtml(booking.booking_status)}
                </div>
                <div class="delete-booking-meta">
                    Customer: ${escapeHtml(customer)}<br>
                    Start: ${escapeHtml(booking.slot_start)}<br>
                    End: ${escapeHtml(booking.slot_end)}<br>
                    Duration: ${escapeHtml(booking.duration_minutes)} min<br>
                    Amount: ${escapeHtml(booking.amount_quoted)} ${escapeHtml(booking.currency)}<br>
                    Payment: ${escapeHtml(booking.payment_status)}<br>
                    Contract: ${escapeHtml(booking.contract_address || '—')}
                </div>
            </div>
        `;
    }).join('');
}

async function runDelete(confirmPaidFutureBookings = false) {
    try {
        setDebugStatus('Deleting profile...');

        if (els.deleteProfileConfirmBtn) els.deleteProfileConfirmBtn.disabled = true;
        if (els.deleteProfileForceBtn) els.deleteProfileForceBtn.disabled = true;

        const result = await deleteExpertProfile(confirmPaidFutureBookings);

        setDebugStatus(
            result.refunds_dispatched
                ? 'Profile deleted. Refund processing started.'
                : 'Profile deleted.'
        );

        window.location.href = result.redirect_to || '/';
    } catch (error) {
        console.error(error);
        setDebugStatus(error.message || 'Delete failed.');
    } finally {
        if (els.deleteProfileConfirmBtn) els.deleteProfileConfirmBtn.disabled = false;
        if (els.deleteProfileForceBtn) els.deleteProfileForceBtn.disabled = false;
    }
}

export function bindDeleteProfileFlow() {
    els.deleteProfileBtn?.addEventListener('click', () => {
        openModal(els.deleteProfileModal);
    });

    els.deleteProfileCancelBtn?.addEventListener('click', () => {
        closeModal(els.deleteProfileModal);
    });

    els.deleteProfilePaidBookingsCancelBtn?.addEventListener('click', () => {
        closeModal(els.deleteProfilePaidBookingsModal);
    });

    els.deleteProfileConfirmBtn?.addEventListener('click', async () => {
        try {
            setDebugStatus('Checking future paid bookings...');
            els.deleteProfileConfirmBtn.disabled = true;

            const preview = await previewDeleteExpertProfile();

            closeModal(els.deleteProfileModal);

            if (preview.has_paid_future_bookings) {
                renderPaidFutureBookings(preview.paid_future_bookings || []);
                openModal(els.deleteProfilePaidBookingsModal);
                setDebugStatus('');
                return;
            }

            await runDelete(false);
        } catch (error) {
            console.error(error);
            setDebugStatus(error.message || 'Delete preview failed.');
        } finally {
            els.deleteProfileConfirmBtn.disabled = false;
        }
    });

    els.deleteProfileForceBtn?.addEventListener('click', async () => {
        await runDelete(true);
    });
}