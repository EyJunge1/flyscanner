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
		class="pointer-events-auto absolute inset-x-0 bottom-0 z-20 border-t border-border/90 bg-bg/95 px-4 py-3 backdrop-blur-md"
	>
		<div class="mx-auto flex max-w-4xl flex-col gap-2">
			<div class="flex items-center justify-between gap-3 text-xs text-muted">
				<span class="font-medium tracking-wide text-warn uppercase">Replay</span>
				<span class="font-mono text-text">{fmt(fleet.replayTs)}</span>
				<button
					type="button"
					class="rounded p-1 text-muted hover:bg-surface-hover hover:text-text-bright"
					onclick={() => fleet.exitReplay()}
					title="Live (Esc)"
				>
					<X size={14} />
				</button>
			</div>

			{#if fleet.replaySnapshots.length === 0}
				<p class="text-center text-xs text-muted">
					Keine Snapshots in den letzten {fleet.replayHours}h — History läuft weiter im Hintergrund.
				</p>
			{:else}
				<input
					type="range"
					min="0"
					max={Math.max(0, fleet.replaySnapshots.length - 1)}
					value={fleet.replayIndex}
					class="w-full accent-accent"
					oninput={onScrub}
				/>
				<div class="flex items-center justify-center gap-2">
					<button
						type="button"
						class="rounded-md border border-border p-1.5 text-text hover:border-accent"
						onclick={() => fleet.applyReplayIndex(fleet.replayIndex - 1)}
						title="←"
					>
						<SkipBack size={14} />
					</button>
					<button
						type="button"
						class="rounded-md border border-border px-3 py-1.5 text-text hover:border-accent"
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
						class="rounded-md border border-border p-1.5 text-text hover:border-accent"
						onclick={() => fleet.applyReplayIndex(fleet.replayIndex + 1)}
						title="→"
					>
						<SkipForward size={14} />
					</button>
					<span class="ml-2 font-mono text-[11px] text-muted">
						{fleet.replayIndex + 1}/{fleet.replaySnapshots.length}
					</span>
				</div>
			{/if}
		</div>
	</div>
{/if}
