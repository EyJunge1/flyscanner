<script lang="ts">
	import { fleet } from '../store.svelte';
</script>

{#if fleet.showHelp}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="bg-bg/70 pointer-events-auto absolute inset-0 z-30 flex items-center justify-center p-4 backdrop-blur-sm"
		onclick={() => (fleet.showHelp = false)}
	>
		<div
			class="border-border bg-surface text-text max-h-[85vh] w-full max-w-md overflow-y-auto rounded-xl border p-5 shadow-2xl"
			onclick={(e) => e.stopPropagation()}
		>
			<div class="mb-3 flex items-center justify-between">
				<h2 class="text-accent text-sm font-semibold tracking-wide uppercase">Shortcuts</h2>
				<button
					type="button"
					class="text-muted hover:text-text text-xs"
					onclick={() => (fleet.showHelp = false)}>Esc</button
				>
			</div>
			<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-xs">
				<dt class="text-accent/90 font-mono">T</dt>
				<dd>Trails (track lifelines)</dd>
				<dt class="text-accent/90 font-mono">L</dt>
				<dd>Labels</dd>
				<dt class="text-accent/90 font-mono">O</dt>
				<dd>Extended labels (alt/speed)</dd>
				<dt class="text-accent/90 font-mono">R</dt>
				<dd>Range rings</dd>
				<dt class="text-accent/90 font-mono">H</dt>
				<dd>Heatmap</dd>
				<dt class="text-accent/90 font-mono">P</dt>
				<dd>pTracks</dd>
				<dt class="text-accent/90 font-mono">F</dt>
				<dd>Follow selected</dd>
				<dt class="text-accent/90 font-mono">G</dt>
				<dd>Home (receiver)</dd>
				<dt class="text-accent/90 font-mono">I</dt>
				<dd>Isolate selected</dd>
				<dt class="text-accent/90 font-mono">M</dt>
				<dd>Cycle basemap</dd>
				<dt class="text-accent/90 font-mono">B</dt>
				<dd>Dim map</dd>
				<dt class="text-accent/90 font-mono">[</dt>
				<dd>Toggle sidebar</dd>
				<dt class="text-accent/90 font-mono">Space</dt>
				<dd>Replay play/pause</dd>
				<dt class="text-accent/90 font-mono">← →</dt>
				<dd>Replay step</dd>
				<dt class="text-accent/90 font-mono">Esc</dt>
				<dd>Exit replay / clear</dd>
				<dt class="text-accent/90 font-mono">?</dt>
				<dd>This help</dd>
			</dl>
		</div>
	</div>
{/if}
