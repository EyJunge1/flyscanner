<script lang="ts">
	import { Copy, Download, X } from '@lucide/svelte';
	import { ageSec, climbArrow, fmtAge } from '../geo';
	import { fleet } from '../store.svelte';
	import type { Aircraft } from '../types';

	let { aircraft: a }: { aircraft: Aircraft } = $props();

	const dist = $derived(fleet.distanceNm(a));
	const seen = $derived(fmtAge(ageSec(a.last_seen)));
	const na = 'n/a';

	function row(label: string, value: string | null | undefined) {
		return { label, value: value && value !== '' ? value : na };
	}
</script>

<div class="rounded-md border border-border bg-surface/90">
	<div class="flex items-start justify-between gap-2 border-b border-border px-2.5 py-2">
		<div class="min-w-0">
			<p class="truncate font-mono text-sm font-semibold text-accent">
				{a.callsign ?? a.registration ?? a.icao24}
			</p>
			<p class="font-mono text-[10px] text-muted uppercase">
				Hex: {a.icao24}
				<button
					type="button"
					class="ml-1 text-accent hover:text-accent"
					onclick={() => fleet.copyLink(a)}
					title="Copy link"
				>
					<Copy size={10} class="inline" />
				</button>
			</p>
		</div>
		<button
			type="button"
			class="shrink-0 rounded p-0.5 text-muted hover:bg-surface-hover hover:text-text"
			onclick={() => fleet.deselect(a.icao24)}
			title="Remove"
		>
			<X size={12} />
		</button>
	</div>

	<div class="space-y-2 px-2.5 py-2 text-[10px]">
		<section>
			<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-muted uppercase">Identity</h3>
			<dl class="grid grid-cols-[5.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-text">
				{#each [row('Reg.', a.registration), row('Airline', a.operator), row('Type', a.typecode), row('Squawk', a.squawk), row('DB flags', 'none')] as r (r.label)}
					<dt class="text-muted">{r.label}</dt>
					<dd>{r.value}</dd>
				{/each}
			</dl>
		</section>

		<section>
			<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-muted uppercase">Spatial</h3>
			<dl class="grid grid-cols-[5.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-text">
				{#each [row('Groundspeed', a.groundspeed != null ? `${a.groundspeed.toFixed(0)} kt` : null), row('Baro. alt', a.altitude != null ? `${climbArrow(a.vertical_rate)} ${a.altitude} ft` : null), row('Vert. Rate', a.vertical_rate != null ? `${a.vertical_rate} ft/min` : null), row('Track', a.track != null ? `${a.track.toFixed(1)}°` : null), row('Heading', a.heading != null ? `${a.heading.toFixed(1)}°` : null), row('Pos.', a.latitude != null && a.longitude != null ? `${a.latitude.toFixed(3)}°, ${a.longitude.toFixed(3)}°` : null), row('Distance', dist != null ? `${dist.toFixed(1)} nmi` : null)] as r (r.label)}
					<dt class="text-muted">{r.label}</dt>
					<dd>{r.value}</dd>
				{/each}
			</dl>
		</section>

		<section>
			<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-muted uppercase">Signal</h3>
			<dl class="grid grid-cols-[5.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-text">
				{#each [row('Source', a.source.toUpperCase()), row('RSSI', a.rssi != null ? `${a.rssi.toFixed(1)}` : null), row('Msgs', String(a.messages)), row('Last Seen', seen), row('Track pts', String(a.track_history.length))] as r (r.label)}
					<dt class="text-muted">{r.label}</dt>
					<dd>{r.value}</dd>
				{/each}
			</dl>
		</section>

		<section>
			<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-muted uppercase">Speed</h3>
			<dl class="grid grid-cols-[5.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-text">
				{#each [row('Ground', a.groundspeed != null ? `${a.groundspeed.toFixed(0)} kt` : null), row('True', na), row('Indicated', na), row('Mach', na)] as r (r.label)}
					<dt class="text-muted">{r.label}</dt>
					<dd>{r.value}</dd>
				{/each}
			</dl>
		</section>

		<section>
			<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-muted uppercase">Altitude</h3>
			<dl class="grid grid-cols-[5.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-text">
				{#each [row('Barometric', a.altitude != null ? `${climbArrow(a.vertical_rate)} ${a.altitude} ft` : null), row('Baro. Rate', a.vertical_rate != null ? `${a.vertical_rate} ft/min` : null), row('Geom. WGS84', na), row('QNH', na)] as r (r.label)}
					<dt class="text-muted">{r.label}</dt>
					<dd>{r.value}</dd>
				{/each}
			</dl>
		</section>

		<section>
			<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-muted uppercase">Accuracy</h3>
			<dl class="grid grid-cols-[5.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-text">
				{#each [row('NACP', na), row('SIL', na), row('NACV', na), row('NICBARO', na)] as r (r.label)}
					<dt class="text-muted">{r.label}</dt>
					<dd>{r.value}</dd>
				{/each}
			</dl>
		</section>

		<div class="flex flex-wrap gap-1.5 pt-1">
			<button
				type="button"
				class="chip"
				onclick={() => fleet.downloadKml(a)}
				title="Export KML"
			>
				<Download size={10} /> KML
			</button>
			<button type="button" class="chip" class:active={fleet.follow} onclick={() => fleet.toggleFollow()}>
				Follow
			</button>
		</div>
	</div>
</div>
