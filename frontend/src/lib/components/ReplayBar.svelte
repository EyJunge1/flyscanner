<script lang="ts">
	import { Pause, Play, SkipBack, SkipForward, X } from '@lucide/svelte';
	import { fleet } from '../store.svelte';

	function fmt(ts: number | null): string {
		if (ts == null) return '—';
		return new Date(ts * 1000).toLocaleString(undefined, {
			month: 'short',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		});
	}

	function onScrub(e: Event) {
		const v = Number((e.currentTarget as HTMLInputElement).value);
		fleet.stopReplayPlayback();
		fleet.applyReplayIndex(v);
	}
</script>

{#if fleet.replayMode}
	<div
		class="border-border/90 bg-bg/95 pointer-events-auto absolute inset-x-0 bottom-0 z-20 border-t px-4 py-3 backdrop-blur-md"
	>
		<div class="mx-auto flex max-w-4xl flex-col gap-2">
			<div class="text-muted flex items-center justify-between gap-3 text-xs">
				<span class="text-warn font-medium tracking-wide uppercase">Replay</span>
				<span class="text-text font-mono">{fmt(fleet.replayTs)}</span>
				<button
					type="button"
					class="text-muted hover:bg-surface-hover hover:text-text-bright rounded p-1"
					onclick={() => fleet.exitReplay()}
					title="Live (Esc)"
				>
					<X size={14} />
				</button>
			</div>

			{#if fleet.replaySnapshots.length === 0}
				<p class="text-muted text-center text-xs">
					No snapshots in the last {fleet.replayHours}h — history keeps collecting in the background.
				</p>
			{:else}
				<input
					type="range"
					min="0"
					max={Math.max(0, fleet.replaySnapshots.length - 1)}
					value={fleet.replayIndex}
					class="accent-accent w-full"
					oninput={onScrub}
				/>
				<div class="flex items-center justify-center gap-2">
					<button
						type="button"
						class="border-border text-text hover:border-accent rounded-md border p-1.5"
						onclick={() => fleet.applyReplayIndex(fleet.replayIndex - 1)}
						title="←"
					>
						<SkipBack size={14} />
					</button>
					<button
						type="button"
						class="border-border text-text hover:border-accent rounded-md border px-3 py-1.5"
						onclick={() => fleet.toggleReplayPlayback()}
						title="Space"
					>
						{#if fleet.replayPlaying}
							<Pause size={14} />
						{:else}
							<Play size={14} />
						{/if}
					</button>
					<button
						type="button"
						class="border-border text-text hover:border-accent rounded-md border p-1.5"
						onclick={() => fleet.applyReplayIndex(fleet.replayIndex + 1)}
						title="→"
					>
						<SkipForward size={14} />
					</button>
					<span class="text-muted ml-2 font-mono text-[11px]">
						{fleet.replayIndex + 1}/{fleet.replaySnapshots.length}
					</span>
				</div>
			{/if}
		</div>
	</div>
{/if}
