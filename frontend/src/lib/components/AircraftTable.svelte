<script lang="ts">
	import { fleet, type SortKey, type TableCol } from '../store.svelte';

	const colMeta: { id: TableCol; label: string; sort: SortKey }[] = [
		{ id: 'icao', label: 'Hex', sort: 'icao' },
		{ id: 'callsign', label: 'Callsign', sort: 'callsign' },
		{ id: 'type', label: 'Type', sort: 'type' },
		{ id: 'alt', label: 'Alt. (ft)', sort: 'alt' },
		{ id: 'spd', label: 'Spd. (kt)', sort: 'spd' },
		{ id: 'track', label: 'Track', sort: 'track' },
		{ id: 'src', label: 'Src', sort: 'src' },
		{ id: 'dist', label: 'Dist', sort: 'dist' },
		{ id: 'rssi', label: 'RSSI', sort: 'rssi' }
	];

	const sources = [
		{ id: '' as const, label: 'All' },
		{ id: 'adsb' as const, label: 'ADS-B' },
		{ id: 'mlat' as const, label: 'MLAT' },
		{ id: 'modes' as const, label: 'Mode-S' },
		{ id: 'tisb' as const, label: 'TIS-B' },
		{ id: 'other' as const, label: 'Other' }
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

<div class="flex min-h-0 flex-1 flex-col">
	<div class="border-border space-y-2 border-b p-2">
		<div class="flex gap-1.5">
			<input
				class="border-border bg-surface focus:border-accent min-w-0 flex-1 rounded border px-2 py-1 text-[11px] outline-none"
				placeholder="Search hex / callsign / reg / type"
				bind:value={fleet.searchQuery}
			/>
			<button type="button" class="chip" onclick={() => fleet.clearSearch()}>Clear</button>
		</div>
		<div class="flex gap-1.5">
			<input
				class="border-border bg-surface focus:border-accent min-w-0 flex-1 rounded border px-2 py-1 text-[11px] outline-none"
				placeholder="Jump: lat, lon or callsign"
				bind:value={fleet.jumpQuery}
				onkeydown={(e) => {
					if (e.key === 'Enter') fleet.jumpToQuery();
				}}
			/>
			<button type="button" class="chip" onclick={() => fleet.jumpToQuery()}>Jump</button>
		</div>
		<div class="flex flex-wrap gap-1">
			{#each sources as s (s.id)}
				<button
					type="button"
					class="chip"
					class:active={fleet.filterSource === s.id}
					onclick={() => (fleet.filterSource = s.id)}
				>
					{s.label}
				</button>
			{/each}
		</div>
		<details class="text-muted text-[10px]">
			<summary class="hover:text-text cursor-pointer">Columns</summary>
			<div class="mt-1 flex flex-wrap gap-x-3 gap-y-1">
				{#each colMeta as c (c.id)}
					<label class="inline-flex items-center gap-1">
						<input type="checkbox" bind:checked={fleet.tableCols[c.id]} />
						{c.label}
					</label>
				{/each}
			</div>
		</details>
		<p class="text-muted text-[10px]">
			Total: {fleet.stats.aircraft_current} · Listed: {fleet.list.length}
		</p>
	</div>

	<div class="min-h-0 flex-1 overflow-auto">
		<table class="w-full border-collapse text-left text-[10px]">
			<thead class="bg-bg sticky top-0 z-1">
				<tr class="border-border text-muted border-b">
					{#each colMeta as c (c.id)}
						{#if fleet.tableCols[c.id]}
							<th class="px-1.5 py-1 font-medium">
								<button
									type="button"
									class="hover:text-accent"
									onclick={() => fleet.toggleSort(c.sort)}
								>
									{c.label}{fleet.sortKey === c.sort ? (fleet.sortDir === 'asc' ? ' ↑' : ' ↓') : ''}
								</button>
							</th>
						{/if}
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each fleet.list as a (a.icao24)}
					<tr
						class="border-border/50 hover:bg-surface-hover/80 cursor-pointer border-b
						{fleet.selected.has(a.icao24) ? 'bg-accent/10' : ''}"
						onclick={(e) => fleet.select(a.icao24, e.shiftKey)}
					>
						{#each colMeta as c (c.id)}
							{#if fleet.tableCols[c.id]}
								<td class="text-text px-1.5 py-1 font-mono">{cell(a, c.id)}</td>
							{/if}
						{/each}
					</tr>
				{:else}
					<tr>
						<td class="text-muted px-3 py-8 text-center" colspan="9">No aircraft</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>
