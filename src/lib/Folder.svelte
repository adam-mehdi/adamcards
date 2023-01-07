<script lang="ts">
	import File from './File.svelte';
    import KaTeXRenderer from './KaTeXRenderer.svelte';
	import {slide} from 'svelte/transition';
	
	export let expanded = false;
	export let name = '';

    type FileSystemObject = {
		type: 'folder' | 'file',
		name: string,
		files?: FileSystemObject[]
	}
	export let files: FileSystemObject[];

	function toggle() {
		expanded = !expanded;
	}
</script>

<span class:expanded on:click={toggle} on:keypress={toggle}>{name}</span>

{#if expanded}
	<ul transition:slide={{duration:300}}>
		{#each files as file}
			<li>
				{#if file.type === 'folder'}
					<svelte:self {...file}/>
				{:else}
					<File {...file}/>
				{/if}
			</li>
		{/each}
	</ul>
{/if}

<style>
	span {
		padding: 0 0 0 1.5em;
		background: url(/folder.svg) 0 0.1em no-repeat;
		background-size: 1em 1em;
		font-weight: bold;
		cursor: pointer;
	}

	.expanded {
		background-image: url(/folder-open.svg);
	}

	ul {
		padding: 0.2em 0 0 0.5em;
		margin: 0 0 0 0.5em;
		list-style: none;
		border-left: 1px solid #eee;
	}

	li {
		padding: 0.2em 0;
	}
</style>
