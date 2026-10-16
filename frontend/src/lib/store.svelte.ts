import { SvelteMap, SvelteSet } from 'svelte/reactivity';
import { haversineNm } from './geo';
import {
	getConfig as simConfig,
	getHeatmap as simHeatmap,
	getPtracks as simPtracks,
	getReplay as simReplay,
	getSeries as simSeries,
	isSim,
	startSim,
	stopSim
} from './sim';
import type {
	Aircraft,
	AppConfig,
	HistorySnapshot,
	StatsSummary,
	TrackPoint,
	WsMessage
} from './types';

export type BasemapId = 'osm' | 'dark' | 'satellite';
export type TableCol = 'icao' | 'callsign' | 'type' | 'alt' | 'spd' | 'track' | 'src' | 'dist' | 'rssi';
export type SortKey = TableCol | 'reg';

function wsUrl(): string {
	if (typeof window === 'undefined') return '';
	const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	return `${proto}//${window.location.host}/ws`;
}

class FleetStore {
	aircraft = $state(new SvelteMap<string, Aircraft>());
	selected = $state(new SvelteSet<string>());
	stats = $state<StatsSummary>({
		messages_total: 0,
		aircraft_current: 0,
		message_rate: 0,
		max_range_nm: 0,
		decode_errors: 0
	});
	config = $state<AppConfig>({ receiver_lat: 52.52, receiver_lon: 13.405 });
	connected = $state(false);
	filterIcao = $state('');
	filterCallsign = $state('');
	filterType = $state('');
	filterReg = $state('');
	filterSquawk = $state('');
	minAlt = $state<number | null>(null);
	maxAlt = $state<number | null>(null);
	minGs = $state<number | null>(null);
	maxGs = $state<number | null>(null);
	maxDistNm = $state<number | null>(null);
	/** '' = all, otherwise matches Aircraft.source */
	filterSource = $state<'' | 'adsb' | 'mlat' | 'modes' | 'tisb' | 'other'>('');
	/** Multi-source filter; empty = use filterSource / all */
	filterSources = $state<Set<string>>(new SvelteSet());
	onlyWithPos = $state(true);
	showTrails = $state(true);
	showLabels = $state(true);
	extendedLabels = $state(false);
	showRangeRings = $state(true);
	showHeatmap = $state(false);
	heatmapPoints = $state<TrackPoint[]>([]);
	ptrackPoints = $state<TrackPoint[]>([]);
	showPtracks = $state(false);
	rateSeries = $state<[number, number][]>([]);
	aircraftSeries = $state<[number, number][]>([]);
	rangeSeries = $state<[number, number][]>([]);
	sidebarTab = $state<'fleet' | 'details' | 'stats'>('fleet');
	showHelp = $state(false);
	sidebarCollapsed = $state(true);
	showFilterPanel = $state(false);
	showBottomTable = $state(true);
	bottomTableTab = $state<'search' | 'filters' | 'columns' | 'stats'>('search');
	bottomTableHeight = $state(220);

	follow = $state(false);
	isolate = $state(false);
	basemap = $state<BasemapId>('osm');
	/** 0 = full bright, 1 = fully dimmed overlay */
	mapDim = $state(0);
	/** Incremented to request map home reset */
	homeTick = $state(0);
	/** Jump target for map pan [lon, lat] or null */
	jumpTarget = $state<[number, number] | null>(null);
	hoveredIcao = $state<string | null>(null);
	/** From URL ?lat&lon&zoom — consumed once by the map */
	initialView = $state<{ lon: number; lat: number; zoom: number } | null>(null);
	/** Live map camera for permalink sync */
	mapView = $state<{ lon: number; lat: number; zoom: number } | null>(null);
	/** ICAOs from ?icao= awaiting aircraft data */
	pendingIcaos = $state<string[]>([]);
	/** ?showTrace=YYYY-MM-DD — day history (API TBD) */
	showTraceDate = $state<string | null>(null);

	searchQuery = $state('');
	jumpQuery = $state('');
	sortKey = $state<SortKey>('icao');
	sortDir = $state<'asc' | 'desc'>('asc');
	tableCols = $state<Record<TableCol, boolean>>({
		icao: true,
		callsign: true,
		type: true,
		alt: true,
		spd: true,
		track: true,
		src: true,
		dist: true,
		rssi: false
	});

	/** When true, live WS fleet updates are ignored (history scrubber owns the map). */
	replayMode = $state(false);
	replaySnapshots = $state<HistorySnapshot[]>([]);
	replayIndex = $state(0);
	replayPlaying = $state(false);
	replayHours = $state(1);
	replayLoading = $state(false);

	#ws: WebSocket | null = null;
	#retry = 1000;
	#replayTimer: ReturnType<typeof setInterval> | null = null;

	get list(): Aircraft[] {
		let rows = [...this.aircraft.values()].filter((a) => this.matchesFilter(a));
		if (this.isolate && this.selected.size > 0) {
			rows = rows.filter((a) => this.selected.has(a.icao24));
		}
		const q = this.searchQuery.trim().toLowerCase();
		if (q) {
			rows = rows.filter((a) => {
				const blob = [
					a.icao24,
					a.callsign ?? '',
					a.registration ?? '',
					a.typecode ?? '',
					a.operator ?? ''
				]
					.join(' ')
					.toLowerCase();
				return blob.includes(q);
			});
		}
		return this.#sortRows(rows);
	}

	get selectedAircraft(): Aircraft | null {
		const first = [...this.selected][0];
		return first ? (this.aircraft.get(first) ?? null) : null;
	}

	get selectedList(): Aircraft[] {
		return [...this.selected]
			.map((icao) => this.aircraft.get(icao))
			.filter((a): a is Aircraft => a != null);
	}

	get hoveredAircraft(): Aircraft | null {
		return this.hoveredIcao ? (this.aircraft.get(this.hoveredIcao) ?? null) : null;
	}

	get followAircraft(): Aircraft | null {
		if (!this.follow) return null;
		return this.selectedAircraft;
	}

	get replayTs(): number | null {
		return this.replaySnapshots[this.replayIndex]?.ts ?? null;
	}

	distanceNm(a: Aircraft): number | null {
		if (a.latitude == null || a.longitude == null) return null;
		return haversineNm(
			this.config.receiver_lat,
			this.config.receiver_lon,
			a.latitude,
			a.longitude
		);
	}

	matchesFilter(a: Aircraft): boolean {
		if (this.onlyWithPos && (a.latitude == null || a.longitude == null)) return false;
		if (this.filterIcao && !a.icao24.includes(this.filterIcao.toLowerCase())) return false;
		if (
			this.filterCallsign &&
			!(a.callsign ?? '').toUpperCase().includes(this.filterCallsign.toUpperCase())
		)
			return false;
		if (
			this.filterType &&
			!(a.typecode ?? '').toUpperCase().includes(this.filterType.toUpperCase())
		)
			return false;
		if (
			this.filterReg &&
			!(a.registration ?? '').toUpperCase().includes(this.filterReg.toUpperCase())
		)
			return false;
		if (this.filterSquawk && !(a.squawk ?? '').includes(this.filterSquawk)) return false;
		if (this.minAlt != null && (a.altitude == null || a.altitude < this.minAlt)) return false;
		if (this.maxAlt != null && (a.altitude == null || a.altitude > this.maxAlt)) return false;
		if (this.minGs != null && (a.groundspeed == null || a.groundspeed < this.minGs)) return false;
		if (this.maxGs != null && (a.groundspeed == null || a.groundspeed > this.maxGs)) return false;
		if (this.filterSources.size > 0) {
			if (!this.filterSources.has(a.source)) return false;
		} else if (this.filterSource && a.source !== this.filterSource) {
			return false;
		}
		if (this.maxDistNm != null) {
			const d = this.distanceNm(a);
			if (d == null || d > this.maxDistNm) return false;
		}
		return true;
	}

	get activeFilterCount(): number {
		let n = 0;
		if (this.filterIcao) n++;
		if (this.filterCallsign) n++;
		if (this.filterType) n++;
		if (this.filterReg) n++;
		if (this.filterSquawk) n++;
		if (this.minAlt != null) n++;
		if (this.maxAlt != null) n++;
		if (this.minGs != null) n++;
		if (this.maxGs != null) n++;
		if (this.maxDistNm != null) n++;
		if (this.filterSource || this.filterSources.size > 0) n++;
		if (this.searchQuery) n++;
		return n;
	}

	resetFilters() {
		this.filterIcao = '';
		this.filterCallsign = '';
		this.filterType = '';
		this.filterReg = '';
		this.filterSquawk = '';
		this.minAlt = null;
		this.maxAlt = null;
		this.minGs = null;
		this.maxGs = null;
		this.maxDistNm = null;
		this.filterSource = '';
		this.filterSources = new SvelteSet();
		this.searchQuery = '';
		this.onlyWithPos = true;
	}

	toggleSource(src: string) {
		const s = new SvelteSet(this.filterSources);
		if (s.has(src)) s.delete(src);
		else s.add(src);
		this.filterSources = s;
		this.filterSource = '';
	}

	selectAllVisible() {
		this.selected = new SvelteSet(this.list.map((a) => a.icao24));
		this.sidebarTab = 'details';
	}

	#sortRows(rows: Aircraft[]): Aircraft[] {
		const dir = this.sortDir === 'asc' ? 1 : -1;
		const key = this.sortKey;
		return [...rows].sort((a, b) => {
			const va = this.#sortVal(a, key);
			const vb = this.#sortVal(b, key);
			if (va == null && vb == null) return 0;
			if (va == null) return 1;
			if (vb == null) return -1;
			if (typeof va === 'number' && typeof vb === 'number') return (va - vb) * dir;
			return String(va).localeCompare(String(vb)) * dir;
		});
	}

	#sortVal(a: Aircraft, key: SortKey): string | number | null {
		switch (key) {
			case 'icao':
				return a.icao24;
			case 'callsign':
				return a.callsign;
			case 'reg':
				return a.registration;
			case 'type':
				return a.typecode;
			case 'alt':
				return a.altitude;
			case 'spd':
				return a.groundspeed;
			case 'track':
				return a.track;
			case 'src':
				return a.source;
			case 'dist':
				return this.distanceNm(a);
			case 'rssi':
				return a.rssi;
			default:
				return a.icao24;
		}
	}

	toggleSort(key: SortKey) {
		if (this.sortKey === key) this.sortDir = this.sortDir === 'asc' ? 'desc' : 'asc';
		else {
			this.sortKey = key;
			this.sortDir = 'asc';
		}
	}

	connect() {
		if (typeof window === 'undefined') return;

		if (isSim()) {
			this.connected = true;
			startSim({
				onUpdate: (aircraft) => {
					if (this.replayMode) return;
					this.#handle({ type: 'update', aircraft });
				},
				onStats: (stats) => {
					this.#handle({
						type: 'stats',
						messages_total: stats.messages_total,
						aircraft_current: stats.aircraft_current,
						message_rate: stats.message_rate,
						max_range_nm: stats.max_range_nm
					});
				}
			});
			return;
		}

		if (
			this.#ws &&
			(this.#ws.readyState === WebSocket.OPEN || this.#ws.readyState === WebSocket.CONNECTING)
		)
			return;

		const ws = new WebSocket(wsUrl());
		this.#ws = ws;

		ws.onopen = () => {
			this.connected = true;
			this.#retry = 1000;
		};

		ws.onclose = () => {
			this.connected = false;
			setTimeout(() => this.connect(), this.#retry);
			this.#retry = Math.min(this.#retry * 1.5, 15000);
		};

		ws.onmessage = (ev) => {
			try {
				const msg = JSON.parse(ev.data) as WsMessage;
				this.#handle(msg);
			} catch {
				/* ignore */
			}
		};
	}

	disconnect() {
		this.stopReplayPlayback();
		if (isSim()) {
			stopSim();
			this.connected = false;
			return;
		}
		if (this.#ws) {
			this.#ws.onclose = null;
			this.#ws.close();
			this.#ws = null;
		}
		this.connected = false;
	}

	#handle(msg: WsMessage) {
		if (msg.type === 'snapshot' || msg.type === 'update') {
			if (this.replayMode) return;
			const next = new SvelteMap<string, Aircraft>();
			for (const a of msg.aircraft) {
				next.set(a.icao24, a);
			}
			this.aircraft = next;
			this.#applyPendingIcaos();
		} else if (msg.type === 'delta') {
			if (this.replayMode) return;
			const next = new SvelteMap(this.aircraft);
			for (const icao of msg.remove) {
				next.delete(icao);
			}
			for (const a of msg.upsert) {
				next.set(a.icao24, a);
			}
			this.aircraft = next;
			this.#applyPendingIcaos();
		} else if (msg.type === 'stats') {
			this.stats = {
				...this.stats,
				messages_total: msg.messages_total,
				aircraft_current: msg.aircraft_current,
				message_rate: msg.message_rate,
				max_range_nm: msg.max_range_nm
			};
		}
	}

	#applyPendingIcaos() {
		if (this.pendingIcaos.length === 0) return;
		const found = this.pendingIcaos.filter((icao) => this.aircraft.has(icao));
		if (found.length === 0) return;
		this.selected = new SvelteSet(found);
		this.sidebarTab = 'details';
		this.pendingIcaos = this.pendingIcaos.filter((icao) => !found.includes(icao));
	}

	select(icao: string, multi = false) {
		const add = multi || (this.sidebarTab === 'details' && this.selected.size > 0);
		if (!add) this.selected = new SvelteSet([icao]);
		else {
			const s = new SvelteSet(this.selected);
			if (s.has(icao)) s.delete(icao);
			else s.add(icao);
			this.selected = s;
		}
		this.sidebarTab = 'details';
	}

	deselect(icao: string) {
		const s = new SvelteSet(this.selected);
		s.delete(icao);
		this.selected = s;
		if (this.selected.size === 0) this.follow = false;
	}

	clearSelection() {
		this.selected = new SvelteSet();
		this.follow = false;
	}

	toggleFollow() {
		if (this.selected.size === 0) return;
		this.follow = !this.follow;
	}

	goHome() {
		this.follow = false;
		this.homeTick += 1;
	}

	cycleBasemap() {
		const order: BasemapId[] = ['osm', 'dark', 'satellite'];
		const i = order.indexOf(this.basemap);
		this.basemap = order[(i + 1) % order.length] ?? 'osm';
	}

	jumpToQuery() {
		const q = this.jumpQuery.trim();
		if (!q) return;
		// "lat, lon" or "lat lon"
		const m = q.match(/^(-?\d+(?:\.\d+)?)\s*[, ]\s*(-?\d+(?:\.\d+)?)$/);
		if (m) {
			const lat = Number(m[1]);
			const lon = Number(m[2]);
			if (Number.isFinite(lat) && Number.isFinite(lon)) {
				this.jumpTarget = [lon, lat];
				this.follow = false;
			}
			return;
		}
		// Match callsign / icao / registration in fleet and center
		const needle = q.toLowerCase();
		const hit = [...this.aircraft.values()].find(
			(a) =>
				a.icao24 === needle ||
				(a.callsign ?? '').toLowerCase() === needle ||
				(a.registration ?? '').toLowerCase() === needle
		);
		if (hit?.latitude != null && hit.longitude != null) {
			this.jumpTarget = [hit.longitude, hit.latitude];
			this.select(hit.icao24);
			this.follow = false;
		}
	}

	clearSearch() {
		this.searchQuery = '';
	}

	exportKml(a: Aircraft): string {
		const name = a.callsign ?? a.icao24;
		const coords = a.track_history
			.map((p) => `${p.lon},${p.lat},${p.alt ?? 0}`)
			.join(' ');
		const pos =
			a.longitude != null && a.latitude != null
				? `<Placemark><name>${name}</name><Point><coordinates>${a.longitude},${a.latitude},${a.altitude ?? 0}</coordinates></Point></Placemark>`
				: '';
		return `<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2"><Document><name>${name}</name>
${pos}
<Placemark><name>${name} track</name><LineString><altitudeMode>absolute</altitudeMode><coordinates>${coords}</coordinates></LineString></Placemark>
</Document></kml>`;
	}

	downloadKml(a: Aircraft) {
		const blob = new Blob([this.exportKml(a)], { type: 'application/vnd.google-earth.kml+xml' });
		const url = URL.createObjectURL(blob);
		const link = document.createElement('a');
		link.href = url;
		link.download = `${a.callsign ?? a.icao24}.kml`;
		link.click();
		URL.revokeObjectURL(url);
	}

	copyLink(a: Aircraft) {
		const url = `${window.location.origin}${window.location.pathname}?icao=${a.icao24}`;
		void navigator.clipboard?.writeText(url);
	}

	async loadConfig() {
		if (isSim()) {
			this.config = simConfig();
			return;
		}
		try {
			const r = await fetch('/api/config');
			if (r.ok) this.config = await r.json();
		} catch {
			/* ignore */
		}
	}

	async loadHeatmap(hours = 24) {
		if (isSim()) {
			this.heatmapPoints = simHeatmap(hours);
			return;
		}
		try {
			const r = await fetch(`/api/history/heatmap?hours=${hours}&max=32000`);
			if (r.ok) this.heatmapPoints = await r.json();
		} catch {
			/* ignore */
		}
	}

	async loadPtracks(hours = 8) {
		if (isSim()) {
			this.ptrackPoints = simPtracks(hours);
			return;
		}
		try {
			const r = await fetch(`/api/history/ptracks?hours=${hours}&interval=15`);
			if (r.ok) this.ptrackPoints = await r.json();
		} catch {
			/* ignore */
		}
	}

	async loadSeries() {
		if (isSim()) {
			const j = simSeries();
			this.rateSeries = j.message_rate;
			this.aircraftSeries = j.aircraft_count;
			this.rangeSeries = j.max_range_nm;
			return;
		}
		try {
			const r = await fetch('/api/stats/series');
			if (r.ok) {
				const j = await r.json();
				this.rateSeries = j.message_rate ?? [];
				this.aircraftSeries = j.aircraft_count ?? [];
				this.rangeSeries = j.max_range_nm ?? [];
			}
		} catch {
			/* ignore */
		}
	}

	async startReplay(hours = this.replayHours) {
		this.replayLoading = true;
		this.stopReplayPlayback();
		try {
			let snaps: HistorySnapshot[] = [];
			if (isSim()) {
				snaps = simReplay(hours);
			} else {
				const r = await fetch(`/api/history/replay?hours=${hours}`);
				if (!r.ok) return;
				const j = (await r.json()) as { snapshots?: HistorySnapshot[] };
				snaps = j.snapshots ?? [];
			}
			this.replaySnapshots = snaps;
			this.replayHours = hours;
			if (snaps.length === 0) {
				this.replayMode = true;
				this.replayIndex = 0;
				this.aircraft = new SvelteMap();
				return;
			}
			this.replayMode = true;
			this.applyReplayIndex(0);
		} catch {
			/* ignore */
		} finally {
			this.replayLoading = false;
		}
	}

	applyReplayIndex(index: number) {
		if (this.replaySnapshots.length === 0) {
			this.replayIndex = 0;
			this.aircraft = new SvelteMap();
			return;
		}
		const i = Math.max(0, Math.min(index, this.replaySnapshots.length - 1));
		this.replayIndex = i;
		const snap = this.replaySnapshots[i];
		if (!snap) return;
		const next = new SvelteMap<string, Aircraft>();
		for (const a of snap.aircraft) {
			next.set(a.icao24, a);
		}
		this.aircraft = next;
	}

	toggleReplayPlayback() {
		if (!this.replayMode || this.replaySnapshots.length === 0) return;
		if (this.replayPlaying) this.stopReplayPlayback();
		else this.startReplayPlayback();
	}

	startReplayPlayback() {
		this.stopReplayPlayback();
		this.replayPlaying = true;
		this.#replayTimer = setInterval(() => {
			if (this.replayIndex >= this.replaySnapshots.length - 1) {
				this.stopReplayPlayback();
				return;
			}
			this.applyReplayIndex(this.replayIndex + 1);
		}, 400);
	}

	stopReplayPlayback() {
		this.replayPlaying = false;
		if (this.#replayTimer != null) {
			clearInterval(this.#replayTimer);
			this.#replayTimer = null;
		}
	}

	exitReplay() {
		this.stopReplayPlayback();
		this.replayMode = false;
		this.replaySnapshots = [];
		this.replayIndex = 0;
	}

	handleHotkey(e: KeyboardEvent) {
		const t = e.target as HTMLElement | null;
		if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) {
			if (e.key === 'Escape') (t as HTMLInputElement).blur?.();
			return;
		}

		switch (e.key) {
			case '?':
				e.preventDefault();
				this.showHelp = !this.showHelp;
				break;
			case 'Escape':
				e.preventDefault();
				if (this.showHelp) this.showHelp = false;
				else if (this.replayMode) this.exitReplay();
				else this.clearSelection();
				break;
			case 't':
			case 'T':
				this.showTrails = !this.showTrails;
				break;
			case 'l':
			case 'L':
				this.showLabels = !this.showLabels;
				break;
			case 'o':
			case 'O':
				this.extendedLabels = !this.extendedLabels;
				if (this.extendedLabels) this.showLabels = true;
				break;
			case 'r':
			case 'R':
				this.showRangeRings = !this.showRangeRings;
				break;
			case 'h':
			case 'H':
				void this.#toggleHeatmapHotkey();
				break;
			case 'p':
			case 'P':
				void this.#togglePtracksHotkey();
				break;
			case 'f':
			case 'F':
				this.toggleFollow();
				break;
			case 'i':
			case 'I':
				this.isolate = !this.isolate;
				break;
			case 'm':
			case 'M':
				this.cycleBasemap();
				break;
			case 'b':
			case 'B':
				this.mapDim = this.mapDim > 0 ? 0 : 0.45;
				break;
			case 'g':
			case 'G':
				this.goHome();
				break;
			case '[':
				this.sidebarCollapsed = !this.sidebarCollapsed;
				break;
			case ' ':
				if (this.replayMode) {
					e.preventDefault();
					this.toggleReplayPlayback();
				}
				break;
			case 'ArrowLeft':
				if (this.replayMode) {
					e.preventDefault();
					this.applyReplayIndex(this.replayIndex - 1);
				}
				break;
			case 'ArrowRight':
				if (this.replayMode) {
					e.preventDefault();
					this.applyReplayIndex(this.replayIndex + 1);
				}
				break;
			default:
				break;
		}
	}

	async #toggleHeatmapHotkey() {
		this.showHeatmap = !this.showHeatmap;
		if (this.showHeatmap && this.heatmapPoints.length === 0) await this.loadHeatmap();
	}

	async #togglePtracksHotkey() {
		this.showPtracks = !this.showPtracks;
		if (this.showPtracks && this.ptrackPoints.length === 0) await this.loadPtracks();
	}
}

export const fleet = new FleetStore();
