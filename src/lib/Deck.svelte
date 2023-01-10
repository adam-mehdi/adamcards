<script lang="ts">
	import type { DeckInfo } from '$lib/types/DecksTypes';
	import CogButton from '$lib/CogButton.svelte';

	export let deck: DeckInfo;
	let deckMenuOpen = false;

	$: {
		if (deckMenuOpen) {
			alert('DECK MENU OPEN!');
		}
	}
</script>

<div class="deck" on:click={() => alert('AHHH')} on:keypress={() => alert('AHHH')}>
	<span class="left">
		{deck.name}
	</span>
	<span class="right">
		<em>{deck.deadline.toLocaleString('en-US')}</em>

		<CogButton bind:open={deckMenuOpen} />
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
