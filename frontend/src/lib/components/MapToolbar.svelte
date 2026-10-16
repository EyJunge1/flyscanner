<script lang="ts">
	import {
		CircleHelp,
		Eye,
		EyeOff,
		Filter,
		Flame,
		History,
		Home,
		Layers,
		Navigation,
		Radio,
		Route,
		Table2,
		Tags
	} from '@lucide/svelte';
	import { fleet } from '../store.svelte';

	async function toggleHeatmap() {
		fleet.showHeatmap = !fleet.showHeatmap;
		if (fleet.showHeatmap && fleet.heatmapPoints.length === 0) await fleet.loadHeatmap();
	}
	async function togglePtracks() {
		fleet.showPtracks = !fleet.showPtracks;
		if (fleet.showPtracks && fleet.ptrackPoints.length === 0) await fleet.loadPtracks();
	}
	async function toggleReplay() {
		if (fleet.replayMode) fleet.exitReplay();
		else await fleet.startReplay(fleet.replayHours);
	}

	const basemapLabel = $derived(
		fleet.basemap === 'osm' ? 'OSM' : fleet.basemap === 'dark' ? 'Dark' : 'Sat'
	);
</script>

<div
	class="pointer-events-auto absolute top-3 left-3 z-20 flex max-w-[min(100%-1.5rem,36rem)] flex-col gap-1.5"
>
	<div class="flex flex-wrap gap-1">
		<button
			type="button"
			class="chip"
			class:active={fleet.showFilterPanel}
			onclick={() => (fleet.showFilterPanel = !fleet.showFilterPanel)}
			title="Super Filter"
		>
			<Filter size={12} />
			Filter{fleet.activeFilterCount ? ` (${fleet.activeFilterCount})` : ''}
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.showBottomTable}
			onclick={() => (fleet.showBottomTable = !fleet.showBottomTable)}
			title="Right fleet panel"
		>
			<Table2 size={12} /> Panel
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.follow}
			onclick={() => fleet.toggleFollow()}
			title="F"
		>
			<Navigation size={12} /> Follow
		</button>
		<button type="button" class="chip" onclick={() => fleet.goHome()} title="G">
			<Home size={12} /> Home
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.isolate}
			onclick={() => (fleet.isolate = !fleet.isolate)}
			title="I"
		>
			{#if fleet.isolate}<Eye size={12} />{:else}<EyeOff size={12} />{/if}
			Isolate
		</button>
	</div>
	<div class="flex flex-wrap gap-1">
		<button
			type="button"
			class="chip"
			class:active={fleet.showTrails}
			onclick={() => (fleet.showTrails = !fleet.showTrails)}
			title="T · Track lifelines"
		>
			<Route size={12} /> Trails
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.showLabels}
			onclick={() => (fleet.showLabels = !fleet.showLabels)}
			title="L"
		>
			Labels
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.extendedLabels}
			onclick={() => {
				fleet.extendedLabels = !fleet.extendedLabels;
				if (fleet.extendedLabels) fleet.showLabels = true;
			}}
			title="O"
		>
			<Tags size={12} /> Ext
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.showRangeRings}
			onclick={() => (fleet.showRangeRings = !fleet.showRangeRings)}
			title="R"
		>
			<Radio size={12} /> Rings
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.showHeatmap}
			onclick={toggleHeatmap}
			title="H"
		>
			<Flame size={12} /> Heat
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.showPtracks}
			onclick={togglePtracks}
			title="P"
		>
			pTracks
		</button>
		<button type="button" class="chip" onclick={() => fleet.cycleBasemap()} title="M">
			<Layers size={12} />
			{basemapLabel}
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.mapDim > 0}
			onclick={() => (fleet.mapDim = fleet.mapDim > 0 ? 0 : 0.45)}
			title="B"
		>
			Dim
		</button>
		<button
			type="button"
			class="chip"
			class:active={fleet.replayMode}
			onclick={toggleReplay}
			disabled={fleet.replayLoading}
			title="Replay"
		>
			<History size={12} />
			{fleet.replayLoading ? '…' : 'Replay'}
		</button>
		<button type="button" class="chip" onclick={() => (fleet.showHelp = !fleet.showHelp)} title="?">
			<CircleHelp size={12} />
		</button>
	</div>
</div>
