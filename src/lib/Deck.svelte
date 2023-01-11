<script lang="ts">
	import type { DeckInfo } from '$lib/types/DecksTypes';
	import CogButton from '$lib/CogButton.svelte';
	import { goto } from '$app/navigation';

	export let deck: DeckInfo;
	export let handleOwnDelete: () => Promise<boolean> | null;

	let deckMenuOpen = false;

	$: {
		if (deckMenuOpen) {
			alert('DECK MENU OPEN!');
		}
	}

	async function handleCogRoute(route: string) {
		try {
			// TODO: make the route the route for this deck
			await goto(route);
			return true;
		} catch {
			return false;
		}
	}

	async function handleEditDeadline() {
		// TODO
	}

	async function handleEditName() {
		// TODO
	}

	let cogMenuOptions = [
		{ name: 'Review', action: async () => await handleCogRoute('/') },
		{ name: 'Edit Deck', action: async () => await handleCogRoute('/') },
		{ name: 'Edit Name', action: handleEditName },
		{ name: 'Edit Deadline', action: handleEditDeadline },
		{ name: 'Delete Deck', action: async () => await handleOwnDelete() }
	];
</script>

<div class="deck" on:click={() => alert('AHHH')} on:keypress={() => alert('AHHH')}>
	<span class="left">
		{deck.name}
	</span>
	<span class="right">
		<em>{deck.deadline ? deck.deadline.toLocaleString('en-US') : 'no deadline'}</em>

		<CogButton options={cogMenuOptions} />
	</span>
</div>

<style>
	.deck {
		/* background-color: blueviolet; */
		width: full;
		display: flex;
		justify-content: space-between;
	}
	.left {
		/* background-color: blue; */
		min-width: fit-content;
	}

	.right {
		display: flex;
		justify-content: space-between;
		/* background-color: red; */
		width: 100%;
		max-width: 300px;
	}
</style>
