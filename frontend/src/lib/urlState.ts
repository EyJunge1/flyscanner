import type { BasemapId } from './store.svelte';
import { fleet } from './store.svelte';

/** Apply tar1090/ADSBX-style query params once at startup. */
export function applyUrlParams(search = window.location.search) {
	const p = new URLSearchParams(search);
	const get = (k: string) => {
		for (const [key, val] of p.entries()) {
			if (key.toLowerCase() === k.toLowerCase()) return val;
		}
		return null;
	};
	const has = (k: string) => {
		for (const key of p.keys()) {
			if (key.toLowerCase() === k.toLowerCase()) return true;
		}
		return false;
	};

	const lat = get('lat');
	const lon = get('lon');
	const zoom = get('zoom');
	if (lat != null && lon != null) {
		const la = Number(lat);
		const lo = Number(lon);
		if (Number.isFinite(la) && Number.isFinite(lo)) {
			fleet.initialView = {
				lon: lo,
				lat: la,
				zoom: zoom != null && Number.isFinite(Number(zoom)) ? Number(zoom) : 8
			};
		}
	} else if (zoom != null && Number.isFinite(Number(zoom))) {
		fleet.initialView = {
			lon: fleet.config.receiver_lon,
			lat: fleet.config.receiver_lat,
			zoom: Number(zoom)
		};
	}

	if (has('enableLabels') || get('enableLabels') === '1') fleet.showLabels = true;
	const ext = get('extendedLabels');
	if (ext != null) {
		fleet.extendedLabels = ext !== '0' && ext !== 'false';
		if (fleet.extendedLabels) fleet.showLabels = true;
	}
	if (has('hideSideBar')) fleet.sidebarCollapsed = true;

	const dim = get('mapDim');
	if (dim != null && Number.isFinite(Number(dim))) {
		fleet.mapDim = Math.min(1, Math.max(0, Number(dim)));
	}

	const base = get('baseMap')?.toLowerCase();
	if (base === 'osm' || base === 'dark' || base === 'satellite') {
		fleet.basemap = base as BasemapId;
	} else if (base === 'carto' || base === 'cartodark') {
		fleet.basemap = 'dark';
	} else if (base === 'esri' || base === 'sat') {
		fleet.basemap = 'satellite';
	}

	const amin = get('filterAltMin');
	const amax = get('filterAltMax');
	if (amin != null && Number.isFinite(Number(amin))) fleet.minAlt = Number(amin);
	if (amax != null && Number.isFinite(Number(amax))) fleet.maxAlt = Number(amax);

	const src = get('filterSources') ?? get('filterSource');
	if (src) {
		const first = src.split(',')[0]?.trim().toLowerCase();
		if (first === 'adsb' || first === 'mlat' || first === 'modes' || first === 'tisb' || first === 'other') {
			fleet.filterSource = first;
		}
	}

	const icaoRaw = get('icao');
	if (icaoRaw) {
		const list = icaoRaw
			.split(',')
			.map((s) => s.trim().toLowerCase())
			.filter(Boolean);
		if (list.length > 0) {
			fleet.pendingIcaos = list;
			if (!has('noIsolation')) fleet.isolate = true;
			fleet.sidebarTab = 'details';
		}
	}

	// showTrace=YYYY-MM-DD → hint for future per-aircraft day replay
	const trace = get('showTrace');
	if (trace) fleet.showTraceDate = trace;
}

let syncTimer: ReturnType<typeof setTimeout> | null = null;

/** Debounced write of current UI state into the address bar (shareable links). */
export function scheduleUrlSync() {
	if (typeof window === 'undefined') return;
	if (syncTimer) clearTimeout(syncTimer);
	syncTimer = setTimeout(writeUrl, 400);
}

function writeUrl() {
	const p = new URLSearchParams();
	const sel = [...fleet.selected];
	if (sel.length) p.set('icao', sel.join(','));
	if (fleet.isolate && sel.length) {
		/* isolate is default with icao in tar1090 */
	} else if (sel.length) {
		p.set('noIsolation', '');
	}
	if (fleet.initialView) {
		/* live center comes from map via fleet.mapView */
	}
	if (fleet.mapView) {
		p.set('lat', fleet.mapView.lat.toFixed(3));
		p.set('lon', fleet.mapView.lon.toFixed(3));
		p.set('zoom', fleet.mapView.zoom.toFixed(1));
	}
	if (fleet.showLabels) p.set('enableLabels', '');
	if (fleet.extendedLabels) p.set('extendedLabels', '1');
	if (fleet.sidebarCollapsed) p.set('hideSideBar', '');
	if (fleet.mapDim > 0) p.set('mapDim', String(fleet.mapDim));
	if (fleet.basemap !== 'osm') p.set('baseMap', fleet.basemap);
	if (fleet.minAlt != null) p.set('filterAltMin', String(fleet.minAlt));
	if (fleet.maxAlt != null) p.set('filterAltMax', String(fleet.maxAlt));
	if (fleet.filterSource) p.set('filterSources', fleet.filterSource);
	if (fleet.showTraceDate) p.set('showTrace', fleet.showTraceDate);

	const qs = p.toString();
	const next = `${window.location.pathname}${qs ? `?${qs}` : ''}`;
	if (next !== `${window.location.pathname}${window.location.search}`) {
		history.replaceState(null, '', next);
	}
}
