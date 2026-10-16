<script lang="ts">
	import { Activity, Filter, Moon, Plane, Sun } from '@lucide/svelte';
	import { fleet } from '../store.svelte';
	import { createTheme } from '../theme.svelte';
	import { toggleTheme } from '../theme';
	import AircraftDetails from './AircraftDetails.svelte';
	import AircraftTable from './AircraftTable.svelte';
	import Chart from './Chart.svelte';

	const themeStore = createTheme();
	const isDark = $derived(themeStore.theme === 'dark');
</script>

<aside
	class="pointer-events-auto flex h-full max-w-[92vw] flex-col border-r border-border/80 bg-bg/90 text-text-bright shadow-xl backdrop-blur-md transition-[width] duration-200
	{fleet.sidebarCollapsed ? 'w-0 overflow-hidden border-0' : 'w-[26rem]'}"
>
	<header class="flex items-center gap-3 border-b border-border px-4 py-3">
		<div class="flex h-9 w-9 items-center justify-center rounded-lg bg-accent/20 text-accent">
			<Plane size={20} />
		</div>
		<div class="min-w-0 flex-1">
			<h1 class="truncate text-lg font-semibold tracking-tight">Flyscanner</h1>
			<p class="flex items-center gap-1.5 text-xs text-muted">
				<span
					class="inline-block h-1.5 w-1.5 rounded-full {fleet.connected
						? 'bg-ok'
						: 'bg-rose-400'}"
				></span>
				{fleet.connected ? 'live' : 'reconnect…'}
				· {fleet.stats.aircraft_current} ac · {fleet.stats.message_rate.toFixed(0)} msg/s
			</p>
		</div>
		<button
			type="button"
			class="shrink-0 rounded-md p-1.5 text-muted hover:bg-surface-hover hover:text-text-bright"
			onclick={() => toggleTheme()}
			aria-label={isDark ? 'Switch to light mode' : 'Switch to dark mode'}
			title={isDark ? 'Light mode' : 'Dark mode'}
		>
			{#if isDark}
				<Moon size={16} />
			{:else}
				<Sun size={16} />
			{/if}
		</button>
	</header>

	<nav class="flex gap-1 border-b border-border p-2">
		{#each [['fleet', 'Fleet'], ['details', 'Details'], ['stats', 'Stats']] as [id, label] (id)}
			<button
				class="flex-1 rounded-md px-2 py-1.5 text-xs font-medium transition
				{fleet.sidebarTab === id
					? 'bg-accent/30 text-accent'
					: 'text-muted hover:bg-surface-hover hover:text-text-bright'}"
				onclick={() => (fleet.sidebarTab = id as typeof fleet.sidebarTab)}
			>
				{label}
			</button>
		{/each}
	</nav>

	{#if fleet.sidebarTab === 'fleet'}
		<div class="space-y-2 border-b border-border p-3">
			<div class="flex items-center gap-2 text-xs text-muted">
				<Filter size={14} /> Filters
			</div>
			<div class="grid grid-cols-2 gap-2">
				<input
					class="rounded-md border border-border bg-surface px-2 py-1.5 text-xs outline-none focus:border-accent"
					placeholder="ICAO"
					bind:value={fleet.filterIcao}
				/>
				<input
					class="rounded-md border border-border bg-surface px-2 py-1.5 text-xs outline-none focus:border-accent"
					placeholder="Callsign"
					bind:value={fleet.filterCallsign}
				/>
				<input
					type="number"
					class="rounded-md border border-border bg-surface px-2 py-1.5 text-xs outline-none focus:border-accent"
					placeholder="Min alt"
					value={fleet.minAlt ?? ''}
					oninput={(e) => {
						const v = (e.currentTarget as HTMLInputElement).value;
						fleet.minAlt = v === '' ? null : Number(v);
					}}
				/>
				<input
					type="number"
					class="rounded-md border border-border bg-surface px-2 py-1.5 text-xs outline-none focus:border-accent"
					placeholder="Max alt"
					value={fleet.maxAlt ?? ''}
					oninput={(e) => {
						const v = (e.currentTarget as HTMLInputElement).value;
						fleet.maxAlt = v === '' ? null : Number(v);
					}}
				/>
			</div>
			{#if fleet.replayMode}
				<label class="flex items-center gap-2 pt-1 text-[10px] text-muted">
					Replay hours
					<input
						type="number"
						min="0.25"
						max="48"
						step="0.25"
						class="w-16 rounded border border-border bg-surface px-1.5 py-0.5 text-xs text-text"
						value={fleet.replayHours}
						onchange={async (e) => {
							const v = Number((e.currentTarget as HTMLInputElement).value);
							if (Number.isFinite(v) && v > 0) await fleet.startReplay(v);
						}}
					/>
				</label>
			{/if}
		</div>
		<AircraftTable />
	{:else if fleet.sidebarTab === 'details'}
		{#if fleet.selectedList.length > 0}
			<div class="min-h-0 flex-1 space-y-2 overflow-y-auto p-2">
				{#each fleet.selectedList as a (a.icao24)}
					<AircraftDetails aircraft={a} />
				{/each}
			</div>
		{:else}
			<AircraftTable />
		{/if}
	{:else}
		<div class="space-y-4 overflow-y-auto p-4">
			<div class="flex items-center gap-2 text-sm text-text">
				<Activity size={16} class="text-accent" /> Live metrics
			</div>
			<div class="grid grid-cols-2 gap-2">
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
			</div>
			<Chart points={fleet.rateSeries} label="Message rate" unit="msg/s" />
			<Chart points={fleet.aircraftSeries} color="var(--ui-muted)" label="Aircraft" unit="ac" />
			<Chart points={fleet.rangeSeries} color="var(--ui-ok)" label="Max range" unit="nm" />
			<button
				type="button"
				class="block w-full rounded-md border border-border px-3 py-2 text-center text-xs text-muted hover:border-accent hover:text-accent"
				onclick={() => window.open('/metrics', '_blank', 'noopener,noreferrer')}
			>
				Prometheus /metrics
			</button>
		</div>
	{/if}
</aside>

<style>
	:global(.chip) {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		border-radius: 9999px;
		border: 1px solid rgb(51 65 85);
		padding: 0.2rem 0.55rem;
		font-size: 10px;
		color: rgb(148 163 184);
		background: rgb(15 23 42 / 0.85);
		backdrop-filter: blur(4px);
	}
	:global(.chip.active) {
		border-color: rgb(8 145 178);
		background: rgb(8 145 178 / 0.25);
		color: rgb(165 243 252);
	}
	:global(.chip:disabled) {
		opacity: 0.5;
	}
	:global(.stat) {
		border-radius: 0.5rem;
		border: 1px solid rgb(30 41 59);
		background: rgb(15 23 42);
		padding: 0.65rem 0.75rem;
	}
	:global(.stat .label) {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: rgb(100 116 139);
	}
	:global(.stat .value) {
		margin-top: 0.15rem;
		font-family: ui-monospace, monospace;
		font-size: 1.1rem;
		font-weight: 600;
		color: rgb(226 232 240);
	}
</style>
