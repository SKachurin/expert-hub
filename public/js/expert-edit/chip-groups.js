export function setChipGroupValues(groupEl, values) {
    const set = new Set(values || []);

    groupEl.querySelectorAll('.check-chip').forEach((chip) => {
        const input = chip.querySelector('input');
        const isActive = set.has(input.value) || set.has(Number(input.value));
        input.checked = isActive;
        chip.classList.toggle('active', isActive);
    });
}

export function getChipGroupValues(groupEl, asNumbers = false) {
    return Array.from(groupEl.querySelectorAll('input:checked')).map((input) =>
        asNumbers ? Number(input.value) : input.value
    );
}

export function bindChipGroup(groupEl) {
    groupEl.querySelectorAll('.check-chip').forEach((chip) => {
        chip.addEventListener('click', () => {
            const input = chip.querySelector('input');
            input.checked = !input.checked;
            chip.classList.toggle('active', input.checked);
        });
    });
}