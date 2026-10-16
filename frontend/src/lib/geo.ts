/** Great-circle distance in nautical miles. */
export function haversineNm(lat1: number, lon1: number, lat2: number, lon2: number): number {
	const R = 3440.065;
	const toRad = Math.PI / 180;
	const dLat = (lat2 - lat1) * toRad;
	const dLon = (lon2 - lon1) * toRad;
	const a =
		Math.sin(dLat / 2) ** 2 +
		Math.cos(lat1 * toRad) * Math.cos(lat2 * toRad) * Math.sin(dLon / 2) ** 2;
	return 2 * R * Math.asin(Math.min(1, Math.sqrt(a)));
}

export function ageSec(lastSeen: number, now = Date.now() / 1000): number {
	return Math.max(0, now - lastSeen);
}

export function fmtAge(sec: number): string {
	if (sec < 60) return `${sec.toFixed(1)} s`;
	if (sec < 3600) return `${(sec / 60).toFixed(1)} min`;
	return `${(sec / 3600).toFixed(1)} h`;
}

export function climbArrow(vr: number | null): string {
	if (vr == null || Math.abs(vr) < 50) return '';
	return vr > 0 ? '▲' : '▼';
}
