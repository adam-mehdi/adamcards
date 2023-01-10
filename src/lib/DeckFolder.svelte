<script lang="ts">
	import type { DeckFolderInfo } from '$lib/types/DecksTypes';
	import Deck from '$lib/Deck.svelte';
	import { dndzone } from 'svelte-dnd-action';
	import { flip } from 'svelte/animate';
	import FolderNameEditable from '$lib/FolderNameEditable.svelte';

	export let folder: DeckFolderInfo;

	$: items = folder.contents ? folder.contents : [];

	function handleSort(e) {
		folder.contents = e.detail.items;
		folder = folder;
	}
</script>

<div
	on:keypress={() => (folder.open = !folder.open)}
	on:click|stopPropagation={() => (folder.open = !folder.open)}
>
	<span class="folder-name-container"><span class="folder-name">{folder.name}</span></span>
	<!-- <FolderNameEditable bind:name={folder.name} /> -->

	{#if folder.contents != null && folder.open}
		<div class="child-area">
			<section
				class="child-dnd-area"
				use:dndzone={{ items }}
				on:consider={handleSort}
				on:finalize={handleSort}
			>
				{#each folder.contents as item, i (item.id)}
					{#if item.type == 'folder'}
						<div class="child-element"><svelte:self bind:folder={item} index={i} /></div>
					{:else}
						<div class="child-element"><Deck bind:deck={item} /></div>
					{/if}
				{/each}
			</section>
		</div>
	{/if}
</div>

<style>
	.child-area {
		margin: 2px 0px 2px 2px;
		padding-left: 1em;
		border-left: 1px solid black;
	}

	.child-dnd-area {
		min-height: 10px;
	}

	.child-element {
		margin: 4px 0px 4px 4px;
		width: full;
		/* background-color: aqua; */
	}
</style>
