<script lang="ts" module>
	import type { ExpressionSpecification, StyleSpecification } from 'maplibre-gl';

	type FeatureCollection = {
		type: 'FeatureCollection';
		features: Array<{
			type: 'Feature';
			properties: Record<string, unknown>;
			geometry: {
				type: string;
				coordinates: number[] | number[][];
			};
		}>;
	};

	function emptyFc(): FeatureCollection {
		return { type: 'FeatureCollection', features: [] };
	}

	function pointsToFc(pts: { lat: number; lon: number }[]): FeatureCollection {
		return {
			type: 'FeatureCollection',
			features: pts.map((p) => ({
				type: 'Feature' as const,
				properties: {},
				geometry: { type: 'Point', coordinates: [p.lon, p.lat] }
			}))
		};
	}

	function circleCoords(center: [number, number], nm: number): [number, number][] {
		const [lon, lat] = center;
		const R = 3440.065;
		const d = nm / R;
		const lat1 = (lat * Math.PI) / 180;
		const lon1 = (lon * Math.PI) / 180;
		const coords: [number, number][] = [];
		for (let i = 0; i <= 64; i++) {
			const brng = (i / 64) * 2 * Math.PI;
			const lat2 = Math.asin(
				Math.sin(lat1) * Math.cos(d) + Math.cos(lat1) * Math.sin(d) * Math.cos(brng)
			);
			const lon2 =
				lon1 +
				Math.atan2(
					Math.sin(brng) * Math.sin(d) * Math.cos(lat1),
					Math.cos(d) - Math.sin(lat1) * Math.sin(lat2)
				);
			coords.push([(lon2 * 180) / Math.PI, (lat2 * 180) / Math.PI]);
		}
		return coords;
	}

	type RasterBasemap = {
		kind: 'raster';
		tiles: string[];
		attribution: string;
	};

	type StyleUrlBasemap = {
		kind: 'style';
		url: string;
	};

	type HybridBasemap = {
		kind: 'hybrid';
		tiles: string[];
		attribution: string;
	};

	const BASEMAPS = {
		osm: {
			kind: 'raster',
			tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'],
			attribution: '© OpenStreetMap'
		},
		dark: {
			kind: 'style',
			url: 'https://tiles.openfreemap.org/styles/dark'
		},
		satellite: {
			kind: 'hybrid',
			tiles: [
				'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'
			],
			attribution: '© Esri © OpenStreetMap'
		}
	} as const satisfies Record<string, RasterBasemap | StyleUrlBasemap | HybridBasemap>;

	/** Place name: prefer local DE/EN, then generic. */
	const PLACE_NAME = ['coalesce', ['get', 'name:de'], ['get', 'name:en'], ['get', 'name']];

	/** Esri imagery + OpenFreeMap vector labels (our styling, not Esri raster chrome). */
	function satelliteHybridStyle(): StyleSpecification {
		return {
			version: 8,
			glyphs: 'https://tiles.openfreemap.org/fonts/{fontstack}/{range}.pbf',
			sources: {
				basemap: {
					type: 'raster',
					tiles: [...BASEMAPS.satellite.tiles],
					tileSize: 256,
					attribution: BASEMAPS.satellite.attribution
				},
				openmaptiles: {
					type: 'vector',
					url: 'https://tiles.openfreemap.org/planet'
				}
			},
			layers: [
				{ id: 'basemap', type: 'raster', source: 'basemap' },
				{
					id: 'boundary-country',
					type: 'line',
					source: 'openmaptiles',
					'source-layer': 'boundary',
					filter: ['all', ['==', ['get', 'admin_level'], 2], ['!=', ['get', 'maritime'], 1]],
					paint: {
						'line-color': '#d4d4d4',
						'line-opacity': 0.5,
						'line-width': ['interpolate', ['linear'], ['zoom'], 2, 0.5, 8, 1.1, 12, 1.6]
					}
				},
				{
					id: 'place-country',
					type: 'symbol',
					source: 'openmaptiles',
					'source-layer': 'place',
					maxzoom: 7,
					filter: ['==', ['get', 'class'], 'country'],
					layout: {
						'text-field': PLACE_NAME as ExpressionSpecification,
						'text-font': ['Noto Sans Regular'],
						'text-size': ['interpolate', ['linear'], ['zoom'], 2, 11, 5, 15],
						'text-transform': 'uppercase',
						'text-letter-spacing': 0.06,
						'text-max-width': 10,
						'text-padding': 6,
						'text-allow-overlap': false
					},
					paint: {
						'text-color': '#ffffff',
						'text-halo-color': '#0a0a0a',
						'text-halo-width': 1.75
					}
				},
				{
					id: 'place-city',
					type: 'symbol',
					source: 'openmaptiles',
					'source-layer': 'place',
					minzoom: 5,
					filter: ['match', ['get', 'class'], ['city', 'town'], true, false],
					layout: {
						'text-field': PLACE_NAME as ExpressionSpecification,
						'text-font': ['Noto Sans Regular'],
						'text-size': ['interpolate', ['linear'], ['zoom'], 5, 11, 10, 14, 14, 16],
						'text-letter-spacing': 0.02,
						'text-max-width': 8,
						'text-padding': 4
					},
					paint: {
						'text-color': '#d4d4d4',
						'text-halo-color': '#0a0a0a',
						'text-halo-width': 1.4
					}
				}
			]
		};
	}
</script>

<script lang="ts">
	import { onMount } from 'svelte';
	import { Map as MapLibreMap, Marker, NavigationControl, type GeoJSONSource } from 'maplibre-gl';
	import '../maplibre';
	import { fleet } from '../store.svelte';
	import { createTheme } from '../theme.svelte';
	import type { Aircraft } from '../types';
	import MapToolbar from './MapToolbar.svelte';

	const themeStore = createTheme();

	let wrapper: HTMLDivElement | undefined = $state();
	let container: HTMLDivElement | undefined = $state();
	let map: MapLibreMap | undefined;
	// DOM marker cache — not reactive UI state
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- side-effect Map for MapLibre markers
	let markers = new Map<string, Marker>();
	let lastHomeTick = 0;
	let lastJump: [number, number] | null = null;
	let lastBasemap = fleet.basemap;

	const RING_SOURCE = 'range-rings';
	const TRAIL_SOURCE = 'trails';
	const HEAT_SOURCE = 'heatmap';
	const PTRACK_SOURCE = 'ptracks';

	/** MapLibre needs resolved CSS colors from `layout.css`. */
	function cssVar(name: string): string {
		return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
	}

	function basemapStyle(id: typeof fleet.basemap): string | StyleSpecification {
		const b = BASEMAPS[id];
		if (b.kind === 'style') return b.url;
		if (b.kind === 'hybrid') return satelliteHybridStyle();

		return {
			version: 8,
			sources: {
				basemap: {
					type: 'raster',
					tiles: [...b.tiles],
					tileSize: 256,
					attribution: b.attribution
				}
			},
			layers: [{ id: 'basemap', type: 'raster', source: 'basemap' }]
		};
	}

	function addOverlayLayers() {
		if (!map) return;
		if (!map.getSource(RING_SOURCE)) {
			map.addSource(RING_SOURCE, { type: 'geojson', data: emptyFc() });
			map.addLayer({
				id: 'range-rings-line',
				type: 'line',
				source: RING_SOURCE,
				paint: { 'line-color': cssVar('--ui-accent'), 'line-width': 1, 'line-opacity': 0.35 }
			});
		}
		if (!map.getSource(TRAIL_SOURCE)) {
			map.addSource(TRAIL_SOURCE, { type: 'geojson', data: emptyFc() });
			map.addLayer({
				id: 'trails-line',
				type: 'line',
				source: TRAIL_SOURCE,
				paint: {
					'line-color': cssVar('--ui-accent'),
					'line-width': 2.5,
					'line-opacity': 0.8
				}
			});
		}
		if (!map.getSource(HEAT_SOURCE)) {
			map.addSource(HEAT_SOURCE, { type: 'geojson', data: emptyFc() });
			map.addLayer({
				id: 'heatmap-layer',
				type: 'circle',
				source: HEAT_SOURCE,
				paint: {
					'circle-radius': 2.5,
					'circle-color': cssVar('--ui-accent'),
					'circle-opacity': 0.35
				},
				layout: { visibility: fleet.showHeatmap ? 'visible' : 'none' }
			});
		}
		if (!map.getSource(PTRACK_SOURCE)) {
			map.addSource(PTRACK_SOURCE, { type: 'geojson', data: emptyFc() });
			map.addLayer({
				id: 'ptracks-layer',
				type: 'circle',
				source: PTRACK_SOURCE,
				paint: {
					'circle-radius': 1.5,
					'circle-color': cssVar('--ui-muted'),
					'circle-opacity': 0.4
				},
				layout: { visibility: fleet.showPtracks ? 'visible' : 'none' }
			});
		}
		updateRings();
		updateTrails(fleet.list);
	}

	function syncOverlayColors() {
		if (!map) return;
		const accent = cssVar('--ui-accent');
		const muted = cssVar('--ui-muted');
		if (map.getLayer('range-rings-line')) {
			map.setPaintProperty('range-rings-line', 'line-color', accent);
		}
		if (map.getLayer('trails-line')) {
			map.setPaintProperty('trails-line', 'line-color', accent);
		}
		if (map.getLayer('heatmap-layer')) {
			map.setPaintProperty('heatmap-layer', 'circle-color', accent);
		}
		if (map.getLayer('ptracks-layer')) {
			map.setPaintProperty('ptracks-layer', 'circle-color', muted);
		}
	}

	onMount(() => {
		if (!wrapper || !container) return;

		const iv = fleet.initialView;
		map = new MapLibreMap({
			container,
			style: basemapStyle(fleet.basemap),
			center: iv ? [iv.lon, iv.lat] : [fleet.config.receiver_lon, fleet.config.receiver_lat],
			zoom: iv?.zoom ?? 8,
			// We own container resizes via ResizeObserver — window listener alone misses sidebar toggles
			// and double-firing with RO causes canvas flicker.
			trackResize: false,
			fadeDuration: 0
		});
		if (iv) fleet.initialView = null;

		map.addControl(new NavigationControl(), 'top-right');

		let resizeRaf = 0;
		const resizeNow = () => {
			if (!map) return;
			map.resize();
		};
		/** One resize per frame — never spam map.resize() (causes canvas flicker). */
		const scheduleResize = () => {
			cancelAnimationFrame(resizeRaf);
			resizeRaf = requestAnimationFrame(resizeNow);
		};
		const ro = new ResizeObserver(() => scheduleResize());
		ro.observe(wrapper);

		const pushView = () => {
			if (!map) return;
			const c = map.getCenter();
			fleet.mapView = { lon: c.lng, lat: c.lat, zoom: map.getZoom() };
		};
		map.on('moveend', pushView);
		map.on('zoomend', pushView);

		map.on('load', () => {
			resizeNow();
			pushView();
		});

		map.on('style.load', () => {
			if (!map) return;
			addOverlayLayers();
			for (const m of markers.values()) m.remove();
			markers.clear();
			syncMarkers(fleet.list, fleet.selected, fleet.showLabels, fleet.extendedLabels);
		});

		return () => {
			ro.disconnect();
			cancelAnimationFrame(resizeRaf);
			for (const m of markers.values()) m.remove();
			markers.clear();
			map?.remove();
			map = undefined;
		};
	});

	$effect(() => {
		const tick = fleet.homeTick;
		if (!map || tick === lastHomeTick) return;
		lastHomeTick = tick;
		map.flyTo({
			center: [fleet.config.receiver_lon, fleet.config.receiver_lat],
			zoom: 8,
			essential: true
		});
		updateRings();
	});

	$effect(() => {
		const target = fleet.jumpTarget;
		if (!map || !target || target === lastJump) return;
		lastJump = target;
		map.flyTo({ center: target, zoom: Math.max(map.getZoom(), 9), essential: true });
		fleet.jumpTarget = null;
	});

	$effect(() => {
		const id = fleet.basemap;
		if (!map || id === lastBasemap) return;
		lastBasemap = id;
		const center = map.getCenter();
		const zoom = map.getZoom();
		map.setStyle(basemapStyle(id));
		map.once('style.load', () => {
			map?.jumpTo({ center, zoom });
		});
	});

	$effect(() => {
		void themeStore.theme;
		syncOverlayColors();
	});

	$effect(() => {
		const ac = fleet.followAircraft;
		if (!map || !ac || ac.latitude == null || ac.longitude == null) return;
		map.setCenter([ac.longitude, ac.latitude]);
	});

	$effect(() => {
		const _lat = fleet.config.receiver_lat;
		const _lon = fleet.config.receiver_lon;
		if (map?.getSource(RING_SOURCE)) updateRings();
		void _lat;
		void _lon;
	});

	$effect(() => {
		const list = fleet.list;
		const selected = fleet.selected;
		const showLabels = fleet.showLabels;
		const ext = fleet.extendedLabels;
		if (!map) return;
		syncMarkers(list, selected, showLabels, ext);
		updateTrails(list);
	});

	$effect(() => {
		const show = fleet.showHeatmap;
		const pts = fleet.heatmapPoints;
		if (!map?.getSource(HEAT_SOURCE)) return;
		(map.getSource(HEAT_SOURCE) as GeoJSONSource).setData(pointsToFc(pts));
		map.setLayoutProperty('heatmap-layer', 'visibility', show ? 'visible' : 'none');
	});

	$effect(() => {
		const show = fleet.showPtracks;
		const pts = fleet.ptrackPoints;
		if (!map?.getSource(PTRACK_SOURCE)) return;
		(map.getSource(PTRACK_SOURCE) as GeoJSONSource).setData(pointsToFc(pts));
		map.setLayoutProperty('ptracks-layer', 'visibility', show ? 'visible' : 'none');
	});

	$effect(() => {
		const show = fleet.showRangeRings;
		if (!map?.getLayer('range-rings-line')) return;
		map.setLayoutProperty('range-rings-line', 'visibility', show ? 'visible' : 'none');
	});

	function labelFor(a: Aircraft, extended: boolean): string {
		const base = a.callsign ?? a.icao24.toUpperCase();
		if (!extended) return base;
		const bits = [base];
		if (a.altitude != null) bits.push(`${a.altitude}`);
		if (a.groundspeed != null) bits.push(`${a.groundspeed.toFixed(0)}kt`);
		return bits.join(' ');
	}

	function syncMarkers(
		list: Aircraft[],
		selected: Set<string>,
		showLabels: boolean,
		extended: boolean
	) {
		if (!map) return;
		// eslint-disable-next-line svelte/prefer-svelte-reactivity -- ephemeral sync set
		const seen = new Set<string>();
		for (const a of list) {
			if (a.latitude == null || a.longitude == null) continue;
			seen.add(a.icao24);
			let marker = markers.get(a.icao24);
			const isSelected = selected.has(a.icao24);
			const rot = a.track ?? a.heading ?? 0;

			if (!marker) {
				const el = document.createElement('div');
				el.className = 'ac-marker';
				el.innerHTML = markerHtml();
				el.onclick = (e: MouseEvent) => {
					e.stopPropagation();
					const icao = el.dataset.icao;
					if (icao) fleet.select(icao, e.shiftKey);
				};
				el.onmouseenter = () => {
					fleet.hoveredIcao = el.dataset.icao ?? null;
				};
				el.onmouseleave = () => {
					if (fleet.hoveredIcao === el.dataset.icao) fleet.hoveredIcao = null;
				};
				marker = new Marker({
					element: el,
					anchor: 'center',
					pitchAlignment: 'viewport',
					rotationAlignment: 'map'
				})
					.setLngLat([a.longitude, a.latitude])
					.addTo(map);
				markers.set(a.icao24, marker);
			} else {
				marker.setLngLat([a.longitude, a.latitude]);
			}

			const el = marker.getElement();
			el.dataset.icao = a.icao24;
			el.style.setProperty('--rot', `${rot}deg`);
			const wrap = el.querySelector('.ac-wrap');
			wrap?.classList.toggle('selected', isSelected);
			const label = el.querySelector('.ac-label') as HTMLSpanElement | null;
			if (label) {
				label.hidden = !showLabels;
				label.textContent = labelFor(a, extended);
			}
		}
		for (const [icao, m] of markers) {
			if (!seen.has(icao)) {
				m.remove();
				markers.delete(icao);
			}
		}
	}

	function updateTrails(list: Aircraft[]) {
		if (!map?.getSource(TRAIL_SOURCE) || !fleet.showTrails) {
			if (map?.getSource(TRAIL_SOURCE)) {
				(map.getSource(TRAIL_SOURCE) as GeoJSONSource).setData(emptyFc());
			}
			return;
		}
		const features = list
			.filter((a) => a.track_history.length >= 2)
			.map((a) => ({
				type: 'Feature' as const,
				properties: { icao: a.icao24 },
				geometry: {
					type: 'LineString' as const,
					coordinates: a.track_history.map((p) => [p.lon, p.lat])
				}
			}));
		(map.getSource(TRAIL_SOURCE) as GeoJSONSource).setData({
			type: 'FeatureCollection',
			features
		});
	}

	function updateRings() {
		if (!map?.getSource(RING_SOURCE)) return;
		const center: [number, number] = [fleet.config.receiver_lon, fleet.config.receiver_lat];
		const rings = [50, 100, 150, 200].map((nm) => ({
			type: 'Feature' as const,
			properties: { nm },
			geometry: {
				type: 'LineString' as const,
				coordinates: circleCoords(center, nm)
			}
		}));
		(map.getSource(RING_SOURCE) as GeoJSONSource).setData({
			type: 'FeatureCollection',
			features: rings
		});
	}

	const PLANE_PATH =
		'M17.8 19.2 16 11l3.5-3.5C21 6 21.5 4 21 3c-1-.5-3 0-4.5 1.5L13 8 4.8 6.2c-.5-.1-.9.1-1.1.5l-.3.5c-.2.5-.1 1 .3 1.3L9 12l-2 3H4l-1 1 3 2 2 3 1-1v-3l3-2 3.5 5.3c.3.4.8.5 1.3.3l.5-.2c.4-.3.6-.7.5-1.2z';

	function markerHtml(): string {
		return `<div class="ac-wrap">
			<svg class="ac-icon" viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
				<path
					d="${PLANE_PATH}"
					fill="currentColor"
					stroke="currentColor"
					stroke-width="1.25"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			</svg>
			<span class="ac-label"></span>
		</div>`;
	}
</script>

<div class="absolute inset-0" bind:this={wrapper}>
	<div class="h-full w-full" bind:this={container}></div>
	{#if fleet.mapDim > 0}
		<div
			class="pointer-events-none absolute inset-0 z-1"
			style="background: rgb(0 0 0 / {fleet.mapDim})"
		></div>
	{/if}
	<MapToolbar />
	{#if fleet.hoveredAircraft}
		{@const h = fleet.hoveredAircraft}
		{@const d = fleet.distanceNm(h)}
		<div
			class="border-border bg-bg/90 text-text pointer-events-none absolute bottom-16 left-1/2 z-10 -translate-x-1/2 rounded-md border px-3 py-1.5 font-mono text-[11px] shadow-lg"
		>
			<span class="text-accent font-semibold">{h.callsign ?? h.icao24}</span>
			<span class="text-muted"> · </span>
			{h.altitude != null ? `${h.altitude} ft` : '—'}
			<span class="text-muted"> · </span>
			{h.groundspeed != null ? `${h.groundspeed.toFixed(0)} kt` : '—'}
			<span class="text-muted"> · </span>
			{h.source}
			{#if d != null}
				<span class="text-muted"> · </span>
				{d.toFixed(1)} nmi
			{/if}
			{#if h.rssi != null}
				<span class="text-muted"> · </span>
				{h.rssi.toFixed(1)} dB
			{/if}
		</div>
	{/if}
</div>

<style>
	:global(.ac-marker) {
		cursor: pointer;
		width: 22px;
		height: 22px;
	}
	:global(.ac-wrap) {
		position: relative;
		width: 22px;
		height: 22px;
		color: var(--ui-surface);
		filter: drop-shadow(0 0 1px rgb(255 255 255 / 0.7)) drop-shadow(0 1px 2px rgb(0 0 0 / 0.35));
	}
	:global(.ac-wrap.selected) {
		color: var(--ui-bg);
		z-index: 10;
		filter: drop-shadow(0 0 0 1.5px var(--ui-accent)) drop-shadow(0 1px 2px rgb(0 0 0 / 0.4));
	}
	:global(.ac-icon) {
		display: block;
		transform: rotate(calc(var(--rot, 0deg) - 45deg));
		transform-origin: center center;
	}
	:global(.ac-label) {
		position: absolute;
		top: calc(100% + 2px);
		left: 50%;
		transform: translateX(-50%);
		padding: 1px 4px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--ui-surface) 75%, transparent);
		color: var(--ui-text-bright);
		font:
			600 10px/1.2 ui-monospace,
			ui-sans-serif,
			system-ui,
			sans-serif;
		white-space: nowrap;
		pointer-events: none;
	}
	:global(.ac-label[hidden]) {
		display: none;
	}
</style>
