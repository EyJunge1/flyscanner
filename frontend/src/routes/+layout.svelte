<script lang="ts">
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import { initTheme } from '#lib/theme';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	initTheme();
</script>

<svelte:head>
	<title>Flyscanner</title>
	<meta name="description" content="Modern ADS-B tracker — Rust + Svelte 5" />
	<link rel="icon" href={favicon} type="image/svg+xml" />
</svelte:head>

{@render children()}
