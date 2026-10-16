<script lang="ts">
	import { fleet } from '../store.svelte';
</script>

{#if fleet.showHelp}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="pointer-events-auto absolute inset-0 z-30 flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-sm"
		onclick={() => (fleet.showHelp = false)}
	>
		<div
			class="max-h-[85vh] w-full max-w-md overflow-y-auto rounded-xl border border-slate-700 bg-slate-900 p-5 text-slate-200 shadow-2xl"
			onclick={(e) => e.stopPropagation()}
		>
			<div class="mb-3 flex items-center justify-between">
				<h2 class="text-sm font-semibold tracking-wide text-cyan-100 uppercase">Shortcuts</h2>
				<button
					type="button"
					class="text-xs text-slate-500 hover:text-slate-300"
					onclick={() => (fleet.showHelp = false)}>Esc</button
				>
			</div>
			<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-xs">
				<dt class="font-mono text-cyan-300/90">T</dt>
				<dd>Trails (track lifelines)</dd>
				<dt class="font-mono text-cyan-300/90">L</dt>
				<dd>Labels</dd>
				<dt class="font-mono text-cyan-300/90">O</dt>
				<dd>Extended labels (alt/speed)</dd>
				<dt class="font-mono text-cyan-300/90">R</dt>
				<dd>Range rings</dd>
				<dt class="font-mono text-cyan-300/90">H</dt>
				<dd>Heatmap</dd>
				<dt class="font-mono text-cyan-300/90">P</dt>
				<dd>pTracks</dd>
				<dt class="font-mono text-cyan-300/90">F</dt>
				<dd>Follow selected</dd>
				<dt class="font-mono text-cyan-300/90">G</dt>
				<dd>Home (receiver)</dd>
				<dt class="font-mono text-cyan-300/90">I</dt>
				<dd>Isolate selected</dd>
				<dt class="font-mono text-cyan-300/90">M</dt>
				<dd>Cycle basemap</dd>
				<dt class="font-mono text-cyan-300/90">B</dt>
				<dd>Dim map</dd>
				<dt class="font-mono text-cyan-300/90">[</dt>
				<dd>Toggle sidebar</dd>
				<dt class="font-mono text-cyan-300/90">Space</dt>
				<dd>Replay play/pause</dd>
				<dt class="font-mono text-cyan-300/90">← →</dt>
				<dd>Replay step</dd>
				<dt class="font-mono text-cyan-300/90">Esc</dt>
				<dd>Exit replay / clear</dd>
				<dt class="font-mono text-cyan-300/90">?</dt>
				<dd>This help</dd>
			</dl>
		</div>
	</div>
{/if}
