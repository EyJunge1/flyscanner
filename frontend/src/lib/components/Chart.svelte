<script lang="ts">
	let {
		points = [],
		color = 'var(--ui-accent)',
		height = 96,
		unit = '',
		label = ''
	}: {
		points?: [number, number][];
		color?: string;
		height?: number;
		unit?: string;
		label?: string;
	} = $props();

	const stats = $derived.by(() => {
		if (points.length === 0) {
			return {
				path: '',
				min: 0,
				max: 0,
				last: null as number | null,
				firstTs: null as number | null,
				lastTs: null as number | null
			};
		}
		const vals = points.map((p) => p[1]);
		const min = Math.min(...vals);
		const max = Math.max(...vals);
		const span = max - min || 1;
		const w = 320;
		const h = height;
		const pad = 4;
		const path = points
			.map((p, i) => {
				const x = (i / Math.max(1, points.length - 1)) * w;
				const y = h - ((p[1] - min) / span) * (h - pad * 2) - pad;
				return `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`;
			})
			.join(' ');
		const last = vals[vals.length - 1] ?? null;
		const firstTs = points[0]?.[0] ?? null;
		const lastTs = points[points.length - 1]?.[0] ?? null;
		return { path, min, max, last, firstTs, lastTs };
	});

	function fmtTs(ts: number | null): string {
		if (ts == null) return '';
		return new Date(ts * 1000).toLocaleTimeString(undefined, {
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<div class="space-y-1">
	<div class="flex items-baseline justify-between gap-2 text-[11px]">
		<span class="text-muted">{label}</span>
		<span class="text-text font-mono">
			{#if stats.last != null}
				{stats.last.toFixed(stats.last >= 100 ? 0 : 1)}{unit ? ` ${unit}` : ''}
			{:else}
				—
			{/if}
		</span>
	</div>
	<svg
		viewBox="0 0 320 {height}"
		class="border-border bg-bg/60 h-24 w-full rounded border"
		preserveAspectRatio="none"
	>
		{#if stats.path}
			<path d={stats.path} fill="none" stroke={color} stroke-width="2" stroke-linejoin="round" />
		{:else}
			<text x="10" y={height / 2} fill="var(--ui-muted)" font-size="11">collecting…</text>
		{/if}
	</svg>
	<div class="text-muted flex justify-between font-mono text-[10px]">
		<span>{fmtTs(stats.firstTs)}</span>
		<span
			>{points.length} pts · {stats.min.toFixed(0)}–{stats.max.toFixed(0)}{unit
				? ` ${unit}`
				: ''}</span
		>
		<span>{fmtTs(stats.lastTs)}</span>
	</div>
</div>
