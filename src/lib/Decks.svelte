<script lang="ts">
	// This files handles:
	// - rendering list of decks
	// - editing deck names
	// - editing deck deadlines
	// - creating a new deck and giving it a name and deadline
	// - deleting a deck
	// - navigating to:
	//   - Editing the deck
	//   - Reviewing the Deck

	import type { DeckFolderInfo, DeckInfo } from '$lib/types/DecksTypes';
	import DeckFolder from '$lib/DeckFolder.svelte';
	import Deck from '$lib/Deck.svelte';
	import { dndzone } from 'svelte-dnd-action';

	let root: DeckFolderInfo | null = {
		id: 2,
		type: 'folder',
		name: 'My Decks',
		contents: [
			{ id: 5, type: 'deck', name: 'German Chapter 1', deadline: new Date() },
			{ id: 7, type: 'deck', name: 'German Chapter 2', deadline: new Date() },
			{
				id: 333,
				type: 'folder',
				name: 'Hello',
				contents: [
					{ id: 10, type: 'deck', name: 'Hello Chapter 1', deadline: new Date() },
					{ id: 11, type: 'deck', name: 'Hello Chapter 2', deadline: new Date() }
				],
				open: true
			}
		],
		open: true
	};
</script>

{#if root}
	<div><DeckFolder bind:folder={root} /></div>
{/if}

<style>
	div {
		list-style-type: none;
		margin: 0px;
		cursor: default;
	}
</style>
