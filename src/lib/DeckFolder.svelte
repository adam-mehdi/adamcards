<script lang="ts">
	import type { DeckFolderInfo } from '$lib/types/DecksTypes';
	import Deck from '$lib/Deck.svelte';
	import { dndzone } from 'svelte-dnd-action';
	import { flip } from 'svelte/animate';
	import FolderNameEditable from '$lib/FolderNameEditable.svelte';
	import { confirm } from '@tauri-apps/api/dialog';

	export let folder: DeckFolderInfo;
	export let handleOwnDelte;

	$: items = folder.contents ? folder.contents : [];

	function handleConsiderSort(e) {
		folder.contents = e.detail.items;
	}
	function handleFinalizeSort(e) {
		folder.contents = e.detail.items;
	}

	async function handleDelete(id: string, type: 'folder' | 'deck', name: string) {
		await confirm(
			`Are you sure you want to delete the ${type} ${name}. Deleting ${name} will delete all of it's contents. `
		).then((response) => {
			if (folder.contents && response) {
				folder.contents = folder.contents.filter((item) => item.id != id);
			}
			return response;
		});
	}
</script>

<div>
	<span
		class="folder-name-container"
		on:keypress={() => (folder.open = !folder.open)}
		on:click|stopPropagation={() => (folder.open = !folder.open)}
		><span class="folder-name">{folder.name}</span></span
	>
	<!-- <FolderNameEditable bind:name={folder.name} /> -->

	{#if folder.open}
		<div class="child-area">
			<section
				class="child-dnd-area"
				use:dndzone={{ items }}
				on:consider={handleConsiderSort}
				on:finalize={handleFinalizeSort}
			>
				{#if folder.contents != null}
					{#each folder.contents as item, i (item.id)}
						{#if item.type == 'folder'}
							<div class="child-element">
								<svelte:self
									bind:folder={item}
									handleOwnDelete={() => handleDelete(item.id, 'deck', item.name)}
								/>
							</div>
						{:else}
							<div class="child-element">
								<Deck
									bind:deck={item}
									handleOwnDelete={() => handleDelete(item.id, 'deck', item.name)}
								/>
							</div>
						{/if}
					{/each}
				{/if}
			</section>
		</div>
	{/if}
</div>

<style>
	.folder-name {
		font-weight: bold;
	}
	.child-area {
		margin: 2px 0px 2px 2px;
		padding-left: 1em;
		border-left: 1px solid black;
	}

	.child-dnd-area {
		min-height: 1.2em;
	}

	.child-element {
		margin: 4px 0px 4px 4px;
		width: full;
		/* background-color: aqua; */
	}
</style>
