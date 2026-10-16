<script lang="ts">
	import { Copy, Download, Navigation, X } from '@lucide/svelte';
	import { ageSec, climbArrow, fmtAge } from '../geo';
	import { fleet } from '../store.svelte';
	import Sparkline from './Sparkline.svelte';

	const a = $derived(fleet.selectedAircraft);
	const dist = $derived(a ? fleet.distanceNm(a) : null);
	const altSeries = $derived.by((): [number, number][] => {
		if (!a) return [];
		return a.track_history
			.filter((p) => p.alt != null)
			.map((p) => [p.ts, p.alt as number] as [number, number]);
	});
</script>

{#if a}
	<div
		class="pointer-events-auto absolute top-14 right-3 z-20 flex max-h-[calc(100%-8rem)] w-[min(20rem,calc(100%-1.5rem))] flex-col overflow-hidden rounded-xl border border-slate-700/80 bg-slate-950/95 shadow-2xl backdrop-blur-md"
	>
		<div class="flex shrink-0 items-start justify-between gap-2 border-b border-slate-800 px-3 py-2.5">
			<div class="min-w-0">
				<p class="truncate font-mono text-base font-semibold text-sky-100">
					{a.callsign ?? a.registration ?? a.icao24}
				</p>
				<p class="font-mono text-[10px] text-slate-500 uppercase">
					Hex: {a.icao24}
					<button
						type="button"
						class="ml-1 inline text-sky-600 hover:text-sky-400"
						onclick={() => fleet.copyLink(a)}
						title="Copy link"
					>
						<Copy size={10} class="inline" />
					</button>
				</p>
			</div>
			<button
				type="button"
				class="rounded p-1 text-slate-500 hover:bg-slate-800"
				onclick={() => fleet.clearSelection()}
			>
				<X size={14} />
			</button>
		</div>

		<div class="min-h-0 flex-1 space-y-3 overflow-y-auto px-3 py-2.5 text-[11px]">
			<section>
				<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-slate-500 uppercase">Identity</h3>
				<dl class="grid grid-cols-[4.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-slate-300">
					<dt class="text-slate-500">Reg.</dt>
					<dd>{a.registration ?? 'n/a'}</dd>
					<dt class="text-slate-500">Airline</dt>
					<dd>{a.operator ?? 'n/a'}</dd>
					<dt class="text-slate-500">Type</dt>
					<dd>{a.typecode ?? 'n/a'}</dd>
					<dt class="text-slate-500">Squawk</dt>
					<dd>{a.squawk ?? 'n/a'}</dd>
				</dl>
			</section>

			<section>
				<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-slate-500 uppercase">
					Altitude lifeline
				</h3>
				{#if altSeries.length >= 2}
					<Sparkline points={altSeries} height={56} />
					<p class="mt-0.5 text-[10px] text-slate-500">
						{altSeries[0]?.[1]} → {altSeries[altSeries.length - 1]?.[1]} ft · {altSeries.length} pts
					</p>
				{:else}
					<p class="text-slate-600">collecting track…</p>
				{/if}
			</section>

			<section>
				<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-slate-500 uppercase">Spatial</h3>
				<dl class="grid grid-cols-[4.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-slate-300">
					<dt class="text-slate-500">GS</dt>
					<dd>{a.groundspeed != null ? `${a.groundspeed.toFixed(0)} kt` : 'n/a'}</dd>
					<dt class="text-slate-500">Alt</dt>
					<dd>
						{a.altitude != null ? `${climbArrow(a.vertical_rate)} ${a.altitude} ft` : 'n/a'}
					</dd>
					<dt class="text-slate-500">VRate</dt>
					<dd>{a.vertical_rate != null ? `${a.vertical_rate} fpm` : 'n/a'}</dd>
					<dt class="text-slate-500">Track</dt>
					<dd>{a.track != null ? `${a.track.toFixed(1)}°` : 'n/a'}</dd>
					<dt class="text-slate-500">Pos</dt>
					<dd>
						{a.latitude != null && a.longitude != null
							? `${a.latitude.toFixed(3)}°, ${a.longitude.toFixed(3)}°`
							: 'n/a'}
					</dd>
					<dt class="text-slate-500">Dist</dt>
					<dd>{dist != null ? `${dist.toFixed(1)} nmi` : 'n/a'}</dd>
				</dl>
			</section>

			<section>
				<h3 class="mb-1 text-[9px] font-semibold tracking-wider text-slate-500 uppercase">Signal</h3>
				<dl class="grid grid-cols-[4.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-slate-300">
					<dt class="text-slate-500">Source</dt>
					<dd>{a.source.toUpperCase()}</dd>
					<dt class="text-slate-500">RSSI</dt>
					<dd>{a.rssi != null ? a.rssi.toFixed(1) : 'n/a'}</dd>
					<dt class="text-slate-500">Msgs</dt>
					<dd>{a.messages}</dd>
					<dt class="text-slate-500">Seen</dt>
					<dd>{fmtAge(ageSec(a.last_seen))}</dd>
				</dl>
			</section>

			<div class="flex flex-wrap gap-1.5 pb-1">
				<button type="button" class="chip" class:active={fleet.follow} onclick={() => fleet.toggleFollow()}>
					<Navigation size={10} /> Follow
				</button>
				<button type="button" class="chip" onclick={() => fleet.downloadKml(a)}>
					<Download size={10} /> KML
				</button>
			</div>
		</div>
	</div>
{/if}
