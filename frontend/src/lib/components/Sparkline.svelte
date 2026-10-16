<script lang="ts">
	let {
		points = [],
		color = '#22d3ee',
		height = 48
	}: {
		points?: [number, number][];
		color?: string;
		height?: number;
	} = $props();

	const path = $derived.by(() => {
		if (points.length < 2) return '';
		const vals = points.map((p) => p[1]);
		const min = Math.min(...vals);
		const max = Math.max(...vals);
		const span = max - min || 1;
		const w = 280;
		const h = height;
		return points
			.map((p, i) => {
				const x = (i / (points.length - 1)) * w;
				const y = h - ((p[1] - min) / span) * (h - 4) - 2;
				return `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`;
			})
			.join(' ');
	});
</script>

<svg viewBox="0 0 280 {height}" class="h-12 w-full" preserveAspectRatio="none">
	{#if path}
		<path d={path} fill="none" stroke={color} stroke-width="2" stroke-linejoin="round" />
	{:else}
		<text x="8" y={height / 2} fill="#64748b" font-size="11">collecting…</text>
	{/if}
</svg>
