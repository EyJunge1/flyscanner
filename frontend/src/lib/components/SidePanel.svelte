<script lang="ts">
	import { ChevronLeft, ChevronRight, Copy, Download, Navigation, X } from '@lucide/svelte';
	import { ageSec, climbArrow, fmtAge } from '../geo';
	import { fleet, type SortKey, type TableCol } from '../store.svelte';
	import Chart from './Chart.svelte';
	import Sparkline from './Sparkline.svelte';

	const a = $derived(fleet.selectedAircraft);
	const dist = $derived(a ? fleet.distanceNm(a) : null);
	const altSeries = $derived.by((): [number, number][] => {
		if (!a) return [];
		return a.track_history
			.filter((p) => p.alt != null)
			.map((p) => [p.ts, p.alt as number] as [number, number]);
	});

	const colMeta: { id: TableCol; label: string; sort: SortKey }[] = [
		{ id: 'icao', label: 'Hex', sort: 'icao' },
		{ id: 'callsign', label: 'Callsign', sort: 'callsign' },
		{ id: 'type', label: 'Type', sort: 'type' },
		{ id: 'alt', label: 'Alt', sort: 'alt' },
		{ id: 'spd', label: 'Spd', sort: 'spd' },
		{ id: 'track', label: 'Trk', sort: 'track' },
		{ id: 'src', label: 'Src', sort: 'src' },
		{ id: 'dist', label: 'Dist', sort: 'dist' },
		{ id: 'rssi', label: 'RSSI', sort: 'rssi' }
	];

	function cell(ac: (typeof fleet.list)[0], id: TableCol): string {
		switch (id) {
			case 'icao':
				return ac.icao24;
			case 'callsign':
				return ac.callsign ?? '—';
			case 'type':
				return ac.typecode ?? '—';
			case 'alt':
				return ac.altitude != null ? String(ac.altitude) : '—';
			case 'spd':
				return ac.groundspeed != null ? ac.groundspeed.toFixed(0) : '—';
			case 'track':
				return ac.track != null ? `${ac.track.toFixed(0)}°` : '—';
			case 'src':
				return ac.source;
			case 'dist': {
				const d = fleet.distanceNm(ac);
				return d != null ? d.toFixed(1) : '—';
			}
			case 'rssi':
				return ac.rssi != null ? ac.rssi.toFixed(1) : '—';
		}
	}

	const width = $derived(fleet.showBottomTable ? '22rem' : '2.25rem');
</script>

<aside
	class="pointer-events-auto absolute top-0 right-0 bottom-0 z-20 flex overflow-hidden border-l border-slate-700/80 bg-slate-950 shadow-2xl"
	style="width: {width}"
>
	<button
		type="button"
		class="flex h-full w-9 shrink-0 flex-col items-center justify-start gap-2 border-r border-slate-800 py-3 text-slate-400 hover:bg-slate-900 hover:text-slate-200"
		onclick={() => (fleet.showBottomTable = !fleet.showBottomTable)}
		title="Toggle panel"
	>
		{#if fleet.showBottomTable}
			<ChevronRight size={16} />
		{:else}
			<ChevronLeft size={16} />
		{/if}
		<span class="rotate-180 text-[10px] tracking-widest uppercase" style="writing-mode: vertical-rl">
			Fleet
		</span>
	</button>

	<div
		class="flex min-w-0 flex-1 flex-col"
		class:invisible={!fleet.showBottomTable}
		class:pointer-events-none={!fleet.showBottomTable}
		aria-hidden={!fleet.showBottomTable}
	>
			<div class="flex shrink-0 flex-wrap items-center gap-1 border-b border-slate-800 px-2 py-1.5">
				{#each (['search', 'filters', 'columns', 'stats'] as const) as tab (tab)}
					<button
						type="button"
						class="rounded px-2 py-1 text-[11px] capitalize
						{fleet.bottomTableTab === tab
							? 'bg-sky-900/40 text-sky-100'
							: 'text-slate-500 hover:text-slate-300'}"
						onclick={() => {
							fleet.bottomTableTab = tab;
							if (tab === 'filters') fleet.showFilterPanel = true;
						}}
					>
						{tab}{tab === 'filters' && fleet.activeFilterCount
							? ` (${fleet.activeFilterCount})`
							: ''}
					</button>
				{/each}
			</div>
			<p class="shrink-0 border-b border-slate-800/60 px-2 py-1 font-mono text-[10px] text-slate-500">
				{fleet.stats.aircraft_current} ac · {fleet.list.length} listed
				{#if fleet.connected}<span class="text-emerald-400"> · live</span>{/if}
				· {fleet.stats.message_rate.toFixed(0)}/s
			</p>

			{#if a}
				<div class="max-h-[42%] shrink-0 overflow-y-auto border-b border-slate-800">
					<div class="flex items-start justify-between gap-2 px-2.5 py-2">
						<div class="min-w-0">
							<p class="truncate font-mono text-sm font-semibold text-sky-100">
								{a.callsign ?? a.registration ?? a.icao24}
							</p>
							<p class="font-mono text-[10px] text-slate-500 uppercase">
								{a.icao24}
								<button
									type="button"
									class="ml-1 text-sky-600 hover:text-sky-400"
									onclick={() => fleet.copyLink(a)}
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
					<div class="space-y-2 px-2.5 pb-2 text-[10px]">
						{#if altSeries.length >= 2}
							<p class="text-[9px] tracking-wider text-slate-500 uppercase">Altitude lifeline</p>
							<Sparkline points={altSeries} height={44} />
						{/if}
						<dl class="grid grid-cols-[3.5rem_1fr] gap-x-2 gap-y-0.5 font-mono text-slate-300">
							<dt class="text-slate-500">Reg</dt>
							<dd>{a.registration ?? 'n/a'}</dd>
							<dt class="text-slate-500">Type</dt>
							<dd>{a.typecode ?? 'n/a'}{a.operator ? ` · ${a.operator}` : ''}</dd>
							<dt class="text-slate-500">Alt</dt>
							<dd>
								{a.altitude != null ? `${climbArrow(a.vertical_rate)} ${a.altitude} ft` : 'n/a'}
							</dd>
							<dt class="text-slate-500">GS</dt>
							<dd>{a.groundspeed != null ? `${a.groundspeed.toFixed(0)} kt` : 'n/a'}</dd>
							<dt class="text-slate-500">Track</dt>
							<dd>{a.track != null ? `${a.track.toFixed(0)}°` : 'n/a'}</dd>
							<dt class="text-slate-500">Dist</dt>
							<dd>{dist != null ? `${dist.toFixed(1)} nmi` : 'n/a'}</dd>
							<dt class="text-slate-500">Src</dt>
							<dd>{a.source} · {fmtAge(ageSec(a.last_seen))}</dd>
						</dl>
						<div class="flex flex-wrap gap-1">
							<button
								type="button"
								class="chip"
								class:active={fleet.follow}
								onclick={() => fleet.toggleFollow()}
							>
								<Navigation size={10} /> Follow
							</button>
							<button type="button" class="chip" onclick={() => fleet.downloadKml(a)}>
								<Download size={10} /> KML
							</button>
						</div>
					</div>
				</div>
			{/if}

			<div class="min-h-0 flex-1 overflow-hidden">
				{#if fleet.bottomTableTab === 'search' || fleet.bottomTableTab === 'columns'}
					{#if fleet.bottomTableTab === 'search'}
						<div class="space-y-1.5 border-b border-slate-800/80 px-2 py-1.5">
							<input
								class="w-full rounded border border-slate-700 bg-slate-900 px-2 py-1 text-[11px] outline-none focus:border-sky-600"
								placeholder="Search…"
								bind:value={fleet.searchQuery}
							/>
							<div class="flex gap-1">
								<input
									class="min-w-0 flex-1 rounded border border-slate-700 bg-slate-900 px-2 py-1 text-[11px] outline-none focus:border-sky-600"
									placeholder="Jump lat,lon / cs"
									bind:value={fleet.jumpQuery}
									onkeydown={(e) => e.key === 'Enter' && fleet.jumpToQuery()}
								/>
								<button type="button" class="chip" onclick={() => fleet.jumpToQuery()}>Go</button>
							</div>
						</div>
					{:else}
						<div
							class="flex flex-wrap gap-x-3 gap-y-1 border-b border-slate-800/80 px-2 py-2 text-[10px] text-slate-400"
						>
							{#each colMeta as c (c.id)}
								<label class="inline-flex items-center gap-1">
									<input type="checkbox" bind:checked={fleet.tableCols[c.id]} />
									{c.label}
								</label>
							{/each}
						</div>
					{/if}
					<div class="h-full overflow-auto">
						<table class="w-full border-collapse text-left text-[10px]">
							<thead class="sticky top-0 bg-slate-950">
								<tr class="border-b border-slate-800 text-slate-500">
									{#each colMeta as c (c.id)}
										{#if fleet.tableCols[c.id]}
											<th class="px-1 py-1">
												<button
													type="button"
													class="hover:text-sky-300"
													onclick={() => fleet.toggleSort(c.sort)}
												>
													{c.label}{fleet.sortKey === c.sort
														? fleet.sortDir === 'asc'
															? ' ↑'
															: ' ↓'
														: ''}
												</button>
											</th>
										{/if}
									{/each}
								</tr>
							</thead>
							<tbody>
								{#each fleet.list as ac (ac.icao24)}
									<tr
										class="cursor-pointer border-b border-slate-800/40 hover:bg-slate-800/50
										{fleet.selected.has(ac.icao24) ? 'bg-sky-950/50' : ''}"
										onclick={(e) => fleet.select(ac.icao24, e.shiftKey)}
									>
										{#each colMeta as c (c.id)}
											{#if fleet.tableCols[c.id]}
												<td class="px-1 py-0.5 font-mono text-slate-300">{cell(ac, c.id)}</td>
											{/if}
										{/each}
									</tr>
								{:else}
									<tr>
										<td class="px-3 py-6 text-center text-slate-500" colspan="9">No aircraft</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else if fleet.bottomTableTab === 'filters'}
					<div class="space-y-2 overflow-y-auto p-3 text-[11px] text-slate-400">
						<button
							type="button"
							class="chip"
							onclick={() => (fleet.showFilterPanel = true)}>Open Super Filter</button
						>
						<p>Active: {fleet.activeFilterCount || 'none'}</p>
						<button type="button" class="chip" onclick={() => fleet.resetFilters()}>Reset</button>
					</div>
				{:else}
					<div class="space-y-3 overflow-y-auto p-3">
						<div class="grid grid-cols-2 gap-2">
							<div class="stat">
								<p class="label">Aircraft</p>
								<p class="value">{fleet.stats.aircraft_current}</p>
							</div>
							<div class="stat">
								<p class="label">Msg/s</p>
								<p class="value">{fleet.stats.message_rate.toFixed(1)}</p>
							</div>
						</div>
						<Chart points={fleet.rateSeries} label="Message rate" unit="msg/s" />
					</div>
				{/if}
			</div>
	</div>
</aside>
