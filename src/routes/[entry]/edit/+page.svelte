<script lang="ts">
	import { flip } from 'svelte/animate'
	import  Search  from '$lib/SearchBarFilter.svelte'
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';

	// load cards from deck
	interface Card {
		id: number,
		front: string,
		back: string,
		last_review: number;
		box_pos: number,
		deck_name: string;
	}
	
	interface CenterPanel {
		front: string,
		back: string,
		prompt: string,
		selected_deck: string,
		multi_card: boolean
	}

	// all decks that are children of the provided file system entry
	interface EntryChildren {
		cards: Card[],
		deck_names: string[]
	}

	/*
	 * Initialize state
	 */

	// init panel
	let panel: CenterPanel = {
		"front": '',
		"back": '',
		"prompt": '',
		"selected_deck": '',
		"multi_card": false,
	};

	// name of folder or deck
	const entry = $page.params.entry;
	
	// load decks that are children of this folder
	let cards: Card[] = [];
	let deck_names: string[] = [];

	async function getDecks() {
		let entityChildren: EntryChildren = await invoke('read_decks', { entry });
		cards = entityChildren.cards;
		deck_names = entityChildren.deck_names;
		panel.selected_deck = deck_names[0];
	} 
	getDecks();

	/*
	 * Button functionality: creating cards, multi-card creation, filtering gallery
	 */

	// number of cards created
	let numCreated = 0;
	// save cards; called on exit (press 'home') or every four cards
	async function saveDecks() {
		await invoke('write_decks', {  cards, numCreated });
	}

	async function createCard() {
		// don't save if either field is empty
		if (panel.front === '' || panel.back === '') {
			return;
		}

		// append to cards
		const id: number = await invoke(
			"calculate_hash", 
			{"deckName": panel.selected_deck, "front": panel.front, "back": panel.back }
			);
		const new_card: Card = { 
			"id": id, 
			"front": panel.front, 
			"back": panel.back, 
			"last_review": 0,
			"box_pos": 0, 
			"deck_name": panel.selected_deck 
		};

		console.log(cards);
		// TODO: crossfade animation and prepend
		cards.splice(0, 0, new_card);
		cards = cards;
		panel.front = '';
		panel.back = '';

		// save  all cards every fourth card made
		if (numCreated % 4 == 0) {
			saveDecks()
		}
		numCreated += 1;
	}

	function toggleMultiCard() {

	}
	
	const filterCards = () => {	
		// return array of cards that match search term
	}


	/*
	 * Animate cards: drag-and-drop and crossfade
	 */

	const dragDuration = 300
	// let cards: number[] = Array(20).fill(1).map((_, i) => i + 1)
	let draggingCard: Card | undefined;
	let animatingCards = new Set()

	function swapWith(card: Card) {
		if (draggingCard === card || animatingCards.has(card)) return
		animatingCards.add(card)
		setTimeout(() => animatingCards.delete(card), dragDuration)
		const cardAIndex = cards.indexOf(draggingCard!)
		const cardBIndex = cards.indexOf(card)
		cards[cardAIndex] = card
		cards[cardBIndex] = draggingCard!
	}

	
</script>
<a href="/" on:click={saveDecks}>Home</a>


<div class="panel">
	<!-- choose deck name; `selected_deck_name` by default -->
	<select name="deck_menu" id="deck_menu">
		{#each deck_names as deck_name}
			<option value={deck_name}> {deck_name} </option>
		{/each}
	</select>

	<!-- show center card field -->
	<div class="panel_card">
			<div class="front">
				<textarea class="panel_text" bind:value={panel.front} />
			</div>
			
		<div class="back">
				<textarea class="panel_text" bind:value={panel.back} />
		</div>
		
		
		<!-- sumbit card or change to multi-card -->
		<div class="submit_bar">
			<button on:click={createCard}> |-> </button>
			<button on:click={toggleMultiCard}> >> </button>
		</div>
		<div class="lookup_bar">
			<Search bind:prompt={panel.prompt} on:input={filterCards} />
		</div>
			
	</div>
</div>

{#if cards.length > 0}
<div class="container">
	{#each cards as card (card)}
		<div
			animate:flip={{ duration: dragDuration }}
			class="card"
			draggable="true"
			on:dragstart={() => draggingCard = card}
			on:dragend={() => draggingCard = undefined}
			on:dragenter={() => swapWith(card)}
			on:dragover|preventDefault
		>
			<div class="front">
				<textarea bind:value={card.front} />
			</div>
			
		<div class="back">
				<textarea bind:value={card.back} />
		</div>
			
		</div>
	{/each}
</div>
{/if}

<style>
	.container {
		display: grid;
/* format depends on size of window		 */
		grid-template-rows: repeat(10, 1fr);
		grid-template-columns: repeat(4, 1fr);
		gap: 8px;
	}

	.card {
		display: flex;
 		flex-direction:column;
		padding: 8px;
		justify-content: center;
		align-items: center;
		
		width: 100%;
		height: 100%;
		font-size: 1.5rem;
	}
	
	.front {
		border-radius: 16px; 
		/* background-color: ivory; */
	}
	
	.back {
		border-radius: 16px; 
		/* background-color: #B9D9EB; */
	}
	
	textarea { 
		width: 256px; 
		height: 64px; 
		border-radius: 16px; 
		resize: None;
		font-size: 12px;
		font-family:"Times";
	}
	

	/* add image support and submit bar at bottom	 */
	.panel {
		height: 364px;
	}
	
	.panel_card {
		display: flex;
 		flex-direction:column;
		padding: 8px;
		justify-content: center;
		align-items: center;
		
		width: 100%;
		height: 100%;
		font-size: 1.5rem;
		
	}
	
	.panel_text {
		width: 312px; 
		height: 86px; 
		border-radius: 16px; 
		resize: None;
		font-size: 16px;
	}
	
	.lookup_bar {
		height: 8px;
	}
	
	button {
		height: 32px;
		font-size: 12px;
	}

	#deck_menu {
		display: flex;
 		flex-direction:column;
		justify-content: center;
		align-items: center;


	}
	
</style>