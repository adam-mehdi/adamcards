<script lang="ts">
	import DeckListDeck from '$lib/DeckListDeck.svelte';

	export let stuff;
	export let thing;
	let isFolder = stuff.deckName === undefined;
	let quagzark = false;
	export let readDeckEntires: () => void;
</script>

{#if isFolder}
	<div class="folder" on:click={() => (quagzark = !quagzark)}><strong>{thing}</strong></div>
	<div class="achild">
		{#if quagzark}
			{#each Object.entries(stuff) as [thing, stuff] (thing)}
				<svelte:self bind:thing bind:stuff bind:readDeckEntires />
			{/each}
		{/if}
	</div>
{:else}
	<DeckListDeck
		bind:deckName={stuff.deckName}
		bind:deckDeadlineDate={stuff.deckDeadlineDate}
		bind:deckDeadlineTime={stuff.deckDeadlineTime}
		bind:readDeckEntires
	/>
{/if}

<style>
	.achild {
		margin-left: 2em;
	}

	.folder {
		user-select: none;
	}
</style>
