// place files you want to import through the `$lib` alias in this folder.

// https://stackoverflow.com/questions/10420352/converting-file-size-in-bytes-to-human-readable-string
function humanSize(bytes, si = false, dp = 0) {
    const thresh = si ? 1000 : 1024;

    if (Math.abs(bytes) < thresh) {
        return bytes + ' B';
    }

    const units = si
        ? ['kB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']
        : ['KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
    let u = -1;
    const r = 10 ** dp;

    do {
        bytes /= thresh;
        ++u;
    } while (Math.round(Math.abs(bytes) * r) / r >= thresh && u < units.length - 1);

    return bytes.toFixed(dp) + ' ' + units[u];
}

function humanPercentage(value) {
    return (value * 100).toFixed(0) + '%';
}

export { humanSize, humanPercentage };