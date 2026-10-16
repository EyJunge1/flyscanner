/**
 * Client-side synthetic traffic for UI development without a backend.
 * Mirrors src/demo.rs + seed_sample_db meta.
 */

import type {
	Aircraft,
	AppConfig,
	HistorySnapshot,
	MessageSource,
	StatsSummary,
	TrackPoint
} from './types';

export const isSim = () => import.meta.env.MODE === 'sim';

const CENTER_LAT = 52.52;
const CENTER_LON = 13.405;
const TRACK_CAP = 120;
const SERIES_CAP = 7200;
const HEATMAP_CAP = 32000;
const SNAPSHOT_INTERVAL_MS = 8000;
const TICK_MS = 500;
const STATS_EVERY_TICKS = 2; // ~1 Hz at 500ms ticks

interface PlaneState {
	icao24: string;
	callsign: string;
	squawk: string;
	lat: number;
	lon: number;
	alt: number;
	track: number;
	gs: number;
	phase: number;
	source: MessageSource;
	registration: string | null;
	typecode: string | null;
	operator: string | null;
	first_seen: number;
	messages: number;
	track_history: TrackPoint[];
}

export type SimHandlers = {
	onUpdate: (aircraft: Aircraft[]) => void;
	onStats: (stats: StatsSummary) => void;
};

let tickTimer: ReturnType<typeof setInterval> | null = null;
let planes: PlaneState[] = [];
let handlers: SimHandlers | null = null;
let tickCount = 0;
let messagesTotal = 0;
let maxRangeNm = 0;
let startedAt = 0;

const rateSeries: [number, number][] = [];
const aircraftSeries: [number, number][] = [];
const rangeSeries: [number, number][] = [];
const heatmapPoints: TrackPoint[] = [];
const replaySnapshots: HistorySnapshot[] = [];
let lastSnapshotAt = 0;

function nowSec(): number {
	return Date.now() / 1000;
}

function randRange(min: number, max: number): number {
	return min + Math.random() * (max - min);
}

function haversineNm(lat1: number, lon1: number, lat2: number, lon2: number): number {
	const R = 3440.065; // Earth radius in NM
	const toRad = Math.PI / 180;
	const dLat = (lat2 - lat1) * toRad;
	const dLon = (lon2 - lon1) * toRad;
	const a =
		Math.sin(dLat / 2) ** 2 +
		Math.cos(lat1 * toRad) * Math.cos(lat2 * toRad) * Math.sin(dLon / 2) ** 2;
	return 2 * R * Math.asin(Math.sqrt(a));
}

function initialPlanes(t0: number): PlaneState[] {
	return [
		{
			icao24: '3c66e6',
			callsign: 'DLH4AK',
			squawk: '1000',
			lat: CENTER_LAT + 0.15,
			lon: CENTER_LON - 0.1,
			alt: 35000,
			track: 90,
			gs: 450,
			phase: 0,
			source: 'adsb',
			registration: 'D-AIMA',
			typecode: 'A320',
			operator: 'Lufthansa',
			first_seen: t0,
			messages: 0,
			track_history: []
		},
		{
			icao24: '4b1804',
			callsign: 'SWR12T',
			squawk: '1000',
			lat: CENTER_LAT - 0.08,
			lon: CENTER_LON + 0.12,
			alt: 28000,
			track: 220,
			gs: 410,
			phase: 1.2,
			source: 'adsb',
			registration: 'HB-JXA',
			typecode: 'BCS3',
			operator: 'Swiss',
			first_seen: t0,
			messages: 0,
			track_history: []
		},
		{
			icao24: 'a37c5a',
			callsign: 'N326SB',
			squawk: '7000',
			lat: CENTER_LAT + 0.02,
			lon: CENTER_LON + 0.03,
			alt: 4500,
			track: 45,
			gs: 120,
			phase: 2.5,
			source: 'adsb',
			registration: 'N326SB',
			typecode: 'C172',
			operator: null,
			first_seen: t0,
			messages: 0,
			track_history: []
		},
		{
			icao24: '4840b2',
			callsign: 'KLM123',
			squawk: '1000',
			lat: CENTER_LAT - 0.2,
			lon: CENTER_LON - 0.15,
			alt: 39000,
			track: 10,
			gs: 480,
			phase: 3.8,
			source: 'mlat',
			registration: null,
			typecode: null,
			operator: null,
			first_seen: t0,
			messages: 0,
			track_history: []
		},
		{
			icao24: '396c00',
			callsign: 'AFR45E',
			squawk: '1000',
			lat: CENTER_LAT + 0.25,
			lon: CENTER_LON + 0.2,
			alt: 32000,
			track: 270,
			gs: 430,
			phase: 5,
			source: 'modes',
			registration: null,
			typecode: null,
			operator: null,
			first_seen: t0,
			messages: 0,
			track_history: []
		}
	];
}

function toAircraft(p: PlaneState, ts: number): Aircraft {
	return {
		icao24: p.icao24,
		first_seen: p.first_seen,
		last_seen: ts,
		callsign: p.callsign,
		squawk: p.squawk,
		altitude: Math.round(p.alt),
		latitude: p.lat,
		longitude: p.lon,
		groundspeed: p.gs,
		track: p.track,
		vertical_rate: Math.round(randRange(-400, 400)),
		heading: p.track,
		rssi: randRange(-35, -15),
		messages: p.messages,
		source: p.source,
		registration: p.registration,
		typecode: p.typecode,
		operator: p.operator,
		track_history: [...p.track_history]
	};
}

function pushCapped<T>(arr: T[], item: T, cap: number) {
	arr.push(item);
	if (arr.length > cap) arr.splice(0, arr.length - cap);
}

function tick() {
	if (!handlers) return;
	const ts = nowSec();
	tickCount += 1;

	for (const p of planes) {
		p.phase += 0.02;
		const rad = (p.track * Math.PI) / 180;
		const nm = (p.gs / 3600) * 0.5;
		const dlat = (nm / 60) * Math.cos(rad);
		const dlon = ((nm / 60) * Math.sin(rad)) / Math.cos((CENTER_LAT * Math.PI) / 180);
		p.lat += dlat;
		p.lon += dlon;
		p.track =
			(((p.track + randRange(-1, 1) + 0.3 * Math.sin(p.phase * 0.1)) % 360) + 360) % 360;
		p.alt = Math.max(1000, Math.min(42000, p.alt + randRange(-50, 50)));
		p.messages += 1;
		messagesTotal += 1;

		const pt: TrackPoint = { ts, lat: p.lat, lon: p.lon, alt: Math.round(p.alt) };
		pushCapped(p.track_history, pt, TRACK_CAP);
		pushCapped(heatmapPoints, pt, HEATMAP_CAP);

		const range = haversineNm(CENTER_LAT, CENTER_LON, p.lat, p.lon);
		if (range > maxRangeNm) maxRangeNm = range;
	}

	const fleet = planes.map((p) => toAircraft(p, ts));
	handlers.onUpdate(fleet);

	if (tickCount % STATS_EVERY_TICKS === 0) {
		const rate = planes.length * (1000 / TICK_MS);
		const stats: StatsSummary = {
			messages_total: messagesTotal,
			aircraft_current: planes.length,
			message_rate: rate,
			max_range_nm: maxRangeNm,
			decode_errors: 0
		};
		handlers.onStats(stats);
		pushCapped(rateSeries, [ts, rate], SERIES_CAP);
		pushCapped(aircraftSeries, [ts, planes.length], SERIES_CAP);
		pushCapped(rangeSeries, [ts, maxRangeNm], SERIES_CAP);
	}

	if (ts - lastSnapshotAt >= SNAPSHOT_INTERVAL_MS / 1000) {
		lastSnapshotAt = ts;
		pushCapped(
			replaySnapshots,
			{
				ts,
				aircraft: fleet.map((a) => ({
					...a,
					track_history: a.track_history.map((t) => ({ ...t }))
				}))
			},
			900
		);
	}
}

export function getConfig(): AppConfig {
	return { receiver_lat: CENTER_LAT, receiver_lon: CENTER_LON };
}

export function getSeries(): {
	message_rate: [number, number][];
	aircraft_count: [number, number][];
	max_range_nm: [number, number][];
} {
	return {
		message_rate: [...rateSeries],
		aircraft_count: [...aircraftSeries],
		max_range_nm: [...rangeSeries]
	};
}

export function getHeatmap(hours = 24, max = 32000): TrackPoint[] {
	const cutoff = nowSec() - hours * 3600;
	const out = heatmapPoints.filter((p) => p.ts >= cutoff).slice(-max);
	return out.map((p) => ({ ...p }));
}

export function getPtracks(hours = 8, intervalSec = 15): TrackPoint[] {
	if (heatmapPoints.length === 0) return [];
	const cutoff = nowSec() - hours * 3600;
	const filtered: TrackPoint[] = [];
	let lastTs = -Infinity;
	for (const p of heatmapPoints) {
		if (p.ts < cutoff) continue;
		if (p.ts - lastTs >= intervalSec) {
			filtered.push({ ...p });
			lastTs = p.ts;
		}
	}
	return filtered;
}

export function getReplay(hours = 1): HistorySnapshot[] {
	const cutoff = nowSec() - hours * 3600;
	return replaySnapshots
		.filter((s) => s.ts >= cutoff)
		.map((s) => ({
			ts: s.ts,
			aircraft: s.aircraft.map((a) => ({
				...a,
				track_history: a.track_history.map((t) => ({ ...t }))
			}))
		}));
}

export function startSim(h: SimHandlers) {
	stopSim();
	handlers = h;
	startedAt = nowSec();
	lastSnapshotAt = startedAt;
	tickCount = 0;
	messagesTotal = 0;
	maxRangeNm = 0;
	rateSeries.length = 0;
	aircraftSeries.length = 0;
	rangeSeries.length = 0;
	heatmapPoints.length = 0;
	replaySnapshots.length = 0;
	planes = initialPlanes(startedAt);
	tick();
	tickTimer = setInterval(tick, TICK_MS);
}

export function stopSim() {
	if (tickTimer != null) {
		clearInterval(tickTimer);
		tickTimer = null;
	}
	handlers = null;
}
