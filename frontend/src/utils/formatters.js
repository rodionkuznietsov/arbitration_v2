export function volumeFormatter(value) {
    if (value >= 1_000_000_000) {
        return (Math.floor(value / 1_000_000_000 * 100) / 100) + 'B';
    }

    if (value >= 1_000_000) {
        return (Math.floor(value / 1_000_000 * 100) / 100) + 'M';
    }

    if (value >= 1_000) {
        return (Math.floor(value / 1_000 * 100) / 100) + 'K';
    }

    if (value >= 0) {
        console.log('Value is less than 1000, returning as is: ' + value);
        return (Math.floor(value / 100 * 100) / 100) + '$';
    }
}

export function formatCurrency(value) {
    if (typeof value != 'number') {
        return value;
    }

    return new Intl.NumberFormat(
        "en-US", {
            minimumFractionDigits: 2,
            maximumFractionDigits: 10,
        }
    ).format(value)
}