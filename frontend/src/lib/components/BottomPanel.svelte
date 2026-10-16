<script lang="ts">
	import { ChevronDown, ChevronUp } from '@lucide/svelte';
	import { fleet, type SortKey, type TableCol } from '../store.svelte';
	import Chart from './Chart.svelte';

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

	function cell(a: (typeof fleet.list)[0], id: TableCol): string {
		switch (id) {
			case 'icao':
				return a.icao24;
			case 'callsign':
				return a.callsign ?? '—';
			case 'type':
				return a.typecode ?? '—';
			case 'alt':
				return a.altitude != null ? String(a.altitude) : '—';
			case 'spd':
				return a.groundspeed != null ? a.groundspeed.toFixed(0) : '—';
			case 'track':
				return a.track != null ? `${a.track.toFixed(0)}°` : '—';
			case 'src':
				return a.source;
			case 'dist': {
				const d = fleet.distanceNm(a);
				return d != null ? d.toFixed(1) : '—';
			}
			case 'rssi':
				return a.rssi != null ? a.rssi.toFixed(1) : '—';
		}
	}
</script>

<div
	class="border-border/80 bg-bg/95 pointer-events-auto absolute right-0 bottom-0 left-0 z-20 flex flex-col border-t shadow-[0_-8px_30px_rgb(0_0_0_/0.35)] backdrop-blur-md"
	style="height: {fleet.showBottomTable ? fleet.bottomTableHeight : 36}px"
>
	<div class="border-border flex h-9 shrink-0 items-center gap-1 border-b px-2">
		<button
			type="button"
			class="text-muted hover:bg-surface-hover rounded p-1"
			onclick={() => (fleet.showBottomTable = !fleet.showBottomTable)}
			title="Toggle table"
		>
			{#if fleet.showBottomTable}
				<ChevronDown size={14} />
			{:else}
				<ChevronUp size={14} />
			{/if}
		</button>
		{#each ['search', 'filters', 'columns', 'stats'] as const as tab (tab)}
			<button
				type="button"
				class="rounded px-2 py-1 text-[11px] capitalize
				{fleet.bottomTableTab === tab ? 'bg-accent-soft text-accent' : 'text-muted hover:text-text'}"
				onclick={() => {
					fleet.bottomTableTab = tab;
					fleet.showBottomTable = true;
					if (tab === 'filters') fleet.showFilterPanel = true;
				}}
			>
				{tab}{tab === 'filters' && fleet.activeFilterCount ? ` (${fleet.activeFilterCount})` : ''}
			</button>
		{/each}
		<span class="text-muted ml-auto font-mono text-[10px]">
			Total: {fleet.stats.aircraft_current} · Listed: {fleet.list.length}
			{#if fleet.connected}
				· <span class="text-ok">live</span>
			{/if}
			· {fleet.stats.message_rate.toFixed(0)} msg/s
		</span>
	</div>

	{#if fleet.showBottomTable}
		<div class="min-h-0 flex-1 overflow-hidden">
			{#if fleet.bottomTableTab === 'search' || fleet.bottomTableTab === 'columns'}
				{#if fleet.bottomTableTab === 'search'}
					<div class="border-border/80 flex gap-2 border-b px-2 py-1.5">
						<input
							class="border-border bg-surface focus:border-accent min-w-0 flex-1 rounded border px-2 py-1 text-[11px] outline-none"
							placeholder="Search…"
							bind:value={fleet.searchQuery}
						/>
						<input
							class="border-border bg-surface focus:border-accent w-44 rounded border px-2 py-1 text-[11px] outline-none"
							placeholder="Jump lat,lon / callsign"
							bind:value={fleet.jumpQuery}
							onkeydown={(e) => e.key === 'Enter' && fleet.jumpToQuery()}
						/>
						<button type="button" class="chip" onclick={() => fleet.jumpToQuery()}>Jump</button>
					</div>
				{:else}
					<div
						class="border-border/80 text-muted flex flex-wrap gap-x-3 gap-y-1 border-b px-3 py-2 text-[10px]"
					>
						{#each colMeta as c (c.id)}
							<label class="inline-flex items-center gap-1">
								<input type="checkbox" bind:checked={fleet.tableCols[c.id]} />
								{c.label}
							</label>
						{/each}
					</div>
				{/if}
				<div
					class="h-full overflow-auto"
					style="max-height: calc({fleet.bottomTableHeight}px - 5rem)"
				>
					<table class="w-full border-collapse text-left text-[10px]">
						<thead class="bg-bg sticky top-0">
							<tr class="border-border text-muted border-b">
								{#each colMeta as c (c.id)}
									{#if fleet.tableCols[c.id]}
										<th class="px-1.5 py-1">
											<button
												type="button"
												class="hover:text-accent"
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
							{#each fleet.list as a (a.icao24)}
								<tr
									class="border-border/40 hover:bg-surface-hover/80 cursor-pointer border-b
									{fleet.selected.has(a.icao24) ? 'bg-accent/15' : ''}"
									onclick={(e) => fleet.select(a.icao24, e.shiftKey)}
								>
									{#each colMeta as c (c.id)}
										{#if fleet.tableCols[c.id]}
											<td class="text-text px-1.5 py-0.5 font-mono">{cell(a, c.id)}</td>
										{/if}
									{/each}
								</tr>
							{:else}
								<tr>
									<td class="text-muted px-3 py-6 text-center" colspan="9">No aircraft</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else if fleet.bottomTableTab === 'filters'}
				<div class="text-muted p-3 text-[11px]">
					<p>
						Super Filter is open on the map
						<button
							type="button"
							class="text-accent hover:underline"
							onclick={() => (fleet.showFilterPanel = true)}>Show filter panel</button
						>
					</p>
					<p class="text-muted mt-2">
						Active filters: {fleet.activeFilterCount || 'none'} · Listed {fleet.list.length}
					</p>
					<button type="button" class="chip mt-2" onclick={() => fleet.resetFilters()}
						>Reset all</button
					>
				</div>
			{:else}
				<div class="grid grid-cols-2 gap-3 overflow-y-auto p-3 md:grid-cols-4">
					<div class="stat">
						<p class="label">Aircraft</p>
						<p class="value">{fleet.stats.aircraft_current}</p>
					</div>
					<div class="stat">
						<p class="label">Msg/s</p>
						<p class="value">{fleet.stats.message_rate.toFixed(1)}</p>
					</div>
					<div class="stat">
						<p class="label">Messages</p>
						<p class="value">{fleet.stats.messages_total}</p>
					</div>
					<div class="stat">
						<p class="label">Max range</p>
						<p class="value">{fleet.stats.max_range_nm.toFixed(0)} nm</p>
					</div>
					<div class="col-span-2 md:col-span-4">
						<Chart points={fleet.rateSeries} label="Message rate" unit="msg/s" />
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>
