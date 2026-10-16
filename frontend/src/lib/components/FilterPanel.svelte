<script lang="ts">
	import { Filter, RotateCcw, X } from '@lucide/svelte';
	import { fleet } from '../store.svelte';

	const sources = ['adsb', 'mlat', 'modes', 'tisb', 'other'] as const;

	function numOrNull(v: string): number | null {
		if (v === '') return null;
		const n = Number(v);
		return Number.isFinite(n) ? n : null;
	}
</script>

{#if fleet.showFilterPanel}
	<div
		class="border-border/80 bg-bg/95 pointer-events-auto absolute top-14 left-3 z-20 w-[min(22rem,calc(100%-1.5rem))] rounded-xl border shadow-2xl backdrop-blur-md"
	>
		<div class="border-border flex items-center justify-between border-b px-3 py-2">
			<div
				class="text-accent flex items-center gap-2 text-xs font-semibold tracking-wide uppercase"
			>
				<Filter size={14} /> Super Filter
				{#if fleet.activeFilterCount > 0}
					<span class="bg-accent/40 text-accent rounded-full px-1.5 py-0.5 text-[10px] normal-case"
						>{fleet.activeFilterCount}</span
					>
				{/if}
			</div>
			<div class="flex items-center gap-1">
				<button
					type="button"
					class="text-muted hover:bg-surface-hover hover:text-text rounded p-1"
					title="Reset"
					onclick={() => fleet.resetFilters()}
				>
					<RotateCcw size={14} />
				</button>
				<button
					type="button"
					class="text-muted hover:bg-surface-hover hover:text-text rounded p-1"
					onclick={() => (fleet.showFilterPanel = false)}
				>
					<X size={14} />
				</button>
			</div>
		</div>

		<div class="max-h-[min(70vh,32rem)] space-y-3 overflow-y-auto p-3 text-[11px]">
			<label class="block space-y-1">
				<span class="text-muted">Search (hex / callsign / reg / type / op)</span>
				<input
					class="border-border bg-surface focus:border-accent w-full rounded-md border px-2 py-1.5 outline-none"
					bind:value={fleet.searchQuery}
					placeholder="e.g. DLH or 3c66"
				/>
			</label>

			<div class="grid grid-cols-2 gap-2">
				<label class="space-y-1">
					<span class="text-muted">Hex</span>
					<input
						class="border-border bg-surface focus:border-accent w-full rounded-md border px-2 py-1.5 outline-none"
						bind:value={fleet.filterIcao}
					/>
				</label>
				<label class="space-y-1">
					<span class="text-muted">Callsign</span>
					<input
						class="border-border bg-surface focus:border-accent w-full rounded-md border px-2 py-1.5 outline-none"
						bind:value={fleet.filterCallsign}
					/>
				</label>
				<label class="space-y-1">
					<span class="text-muted">Type</span>
					<input
						class="border-border bg-surface focus:border-accent w-full rounded-md border px-2 py-1.5 outline-none"
						bind:value={fleet.filterType}
						placeholder="A320"
					/>
				</label>
				<label class="space-y-1">
					<span class="text-muted">Reg</span>
					<input
						class="border-border bg-surface focus:border-accent w-full rounded-md border px-2 py-1.5 outline-none"
						bind:value={fleet.filterReg}
					/>
				</label>
				<label class="space-y-1">
					<span class="text-muted">Squawk</span>
					<input
						class="border-border bg-surface focus:border-accent w-full rounded-md border px-2 py-1.5 outline-none"
						bind:value={fleet.filterSquawk}
						placeholder="7700"
					/>
				</label>
			</div>

			<div>
				<p class="text-muted mb-1.5">Altitude (ft)</p>
				<div class="grid grid-cols-2 gap-2">
					<input
						type="number"
						class="border-border bg-surface focus:border-accent rounded-md border px-2 py-1.5 outline-none"
						placeholder="Min"
						value={fleet.minAlt ?? ''}
						oninput={(e) => (fleet.minAlt = numOrNull(e.currentTarget.value))}
					/>
					<input
						type="number"
						class="border-border bg-surface focus:border-accent rounded-md border px-2 py-1.5 outline-none"
						placeholder="Max"
						value={fleet.maxAlt ?? ''}
						oninput={(e) => (fleet.maxAlt = numOrNull(e.currentTarget.value))}
					/>
				</div>
				<input
					type="range"
					min="0"
					max="45000"
					step="500"
					class="accent-accent mt-2 w-full"
					value={fleet.maxAlt ?? 45000}
					oninput={(e) => {
						fleet.maxAlt = Number(e.currentTarget.value);
					}}
				/>
			</div>

			<div>
				<p class="text-muted mb-1.5">Groundspeed (kt)</p>
				<div class="grid grid-cols-2 gap-2">
					<input
						type="number"
						class="border-border bg-surface focus:border-accent rounded-md border px-2 py-1.5 outline-none"
						placeholder="Min"
						value={fleet.minGs ?? ''}
						oninput={(e) => (fleet.minGs = numOrNull(e.currentTarget.value))}
					/>
					<input
						type="number"
						class="border-border bg-surface focus:border-accent rounded-md border px-2 py-1.5 outline-none"
						placeholder="Max"
						value={fleet.maxGs ?? ''}
						oninput={(e) => (fleet.maxGs = numOrNull(e.currentTarget.value))}
					/>
				</div>
			</div>

			<label class="block space-y-1">
				<span class="text-muted">Max distance (nmi)</span>
				<input
					type="number"
					class="border-border bg-surface focus:border-accent w-full rounded-md border px-2 py-1.5 outline-none"
					placeholder="e.g. 100"
					value={fleet.maxDistNm ?? ''}
					oninput={(e) => (fleet.maxDistNm = numOrNull(e.currentTarget.value))}
				/>
			</label>

			<div>
				<p class="text-muted mb-1.5">Source</p>
				<div class="flex flex-wrap gap-1">
					{#each sources as s (s)}
						<button
							type="button"
							class="chip"
							class:active={fleet.filterSources.has(s) || fleet.filterSource === s}
							onclick={() => fleet.toggleSource(s)}
						>
							{s.toUpperCase()}
						</button>
					{/each}
				</div>
			</div>

			<label class="text-muted flex items-center gap-2">
				<input type="checkbox" bind:checked={fleet.onlyWithPos} />
				Only with position
			</label>

			<div class="border-border flex flex-wrap gap-1.5 border-t pt-2">
				<button
					type="button"
					class="chip"
					class:active={fleet.isolate}
					onclick={() => (fleet.isolate = !fleet.isolate)}>Isolate selected</button
				>
				<button type="button" class="chip" onclick={() => fleet.selectAllVisible()}
					>Select all listed</button
				>
				<button type="button" class="chip" onclick={() => fleet.clearSelection()}
					>Clear selection</button
				>
			</div>

			<p class="text-muted text-[10px]">
				Showing {fleet.list.length} / {fleet.stats.aircraft_current} aircraft
			</p>
		</div>
	</div>
{/if}
