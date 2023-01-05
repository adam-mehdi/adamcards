<script lang="ts">
	//export let input: string = String.raw`Let $x \in X$ be given such that $f(s) \in Y$`;
	export let input: string = 'Hello world';
	import katex from 'katex';

	$: output = input
		.replace(/\$\$(.+?)\$\$/g, (match: string, math: string) => {
			return katex.renderToString(math, { displayMode: true, throwOnError: false });
			// return `<KaTeX math={"${p1}"} displayMath />`;
		})
		.replace(/\$(.+?)\$/g, (match: string, math: string) => {
			return katex.renderToString(math, { displayMode: false, throwOnError: false });
		});
</script>

<svelte:head>
	<link
		rel="stylesheet"
		href="https://cdn.jsdelivr.net/npm/katex@0.12.0/dist/katex.min.css"
		integrity="sha384-AfEj0r4/OFrOo5t7NnNe46zW/tFgW6x/bCJG8FqQCEo3+Aro6EYUG4+cU+KJWu/X"
		crossorigin="anonymous"
	/>
</svelte:head>

{@html output}
