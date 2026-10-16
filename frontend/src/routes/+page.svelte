<script lang="ts">
	import { onMount } from 'svelte';
	import AircraftMap from '#lib/components/AircraftMap.svelte';
	import FilterPanel from '#lib/components/FilterPanel.svelte';
	import HelpOverlay from '#lib/components/HelpOverlay.svelte';
	import ReplayBar from '#lib/components/ReplayBar.svelte';
	import SidePanel from '#lib/components/SidePanel.svelte';
	import { fleet } from '#lib/store.svelte';
	import { applyUrlParams, scheduleUrlSync } from '#lib/urlState';

	onMount(() => {
		applyUrlParams();
		fleet.loadConfig();
		fleet.connect();
		const onKey = (e: KeyboardEvent) => fleet.handleHotkey(e);
		window.addEventListener('keydown', onKey);
		const id = setInterval(() => fleet.loadSeries(), 5000);
		return () => {
			window.removeEventListener('keydown', onKey);
			clearInterval(id);
			fleet.disconnect();
		};
	});

	$effect(() => {
		void fleet.selected;
		void fleet.isolate;
		void fleet.showLabels;
		void fleet.extendedLabels;
		void fleet.sidebarCollapsed;
		void fleet.mapDim;
		void fleet.basemap;
		void fleet.minAlt;
		void fleet.maxAlt;
		void fleet.filterSource;
		void fleet.mapView;
		void fleet.showTraceDate;
		scheduleUrlSync();
	});
</script>

<div class="relative h-dvh w-dvw overflow-hidden bg-slate-950 text-slate-100">
	<main class="absolute inset-0">
		<div class="absolute inset-0">
			<AircraftMap />
			{#if fleet.replayMode}
				<div
					class="pointer-events-none absolute top-3 left-1/2 z-30 -translate-x-1/2 rounded-full border border-amber-700/50 bg-amber-950/80 px-3 py-1 text-[11px] font-medium tracking-wide text-amber-100 uppercase"
				>
					Replay · live paused
				</div>
			{/if}
			<FilterPanel />
			<ReplayBar />
			<HelpOverlay />
		</div>
		<SidePanel />
	</main>
</div>
