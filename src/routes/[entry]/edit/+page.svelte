<script lang="ts">
	import { flip } from 'svelte/animate'
	import  Search  from '$lib/SearchBarFilter.svelte'
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';

	// contains state for the central panel on which cards are created
	interface CenterPanel {
		front: string,
		back: string,
		prompt: string,
		selected_deck: string,
		display_multi: boolean,
		making_multi: boolean,
		textfield: string,
	}

	// rendering data that is extracted from textfield
	interface FieldPair {
		front: string,
		back: string
	}


	// FrontendCard contains properties that can be edited in the frontend
	interface FrontendCard {
		id: number,
		front: string,
		back: string,
		deck_name: string,
	}

	// MetaData contains properties that are read-oly fron the frontend
	interface MetaData {
		is_created: boolean;
		last_review: string,
		box_pos: number,
	}

	// Card tracks both frontend fields and data for backend algorithm and analysis
	interface Card {
		fcard: FrontendCard,
		md: MetaData
	}
	
	// EntryChildren is what is retrieved from the backend
	interface EntryChildren {
		cards: Card[],
		deck_names: string[]
	}

	interface CardState {
		"card_map": Map<number, Card>, // state of cards, mapped by ids
		"fcards": FrontendCard[],      // frontend cards to display in gallery
		"rm_stack": Card[] 		       // contains stack of removed cards for undo
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
		"display_multi": false,
		"making_multi": false,
		"textfield": '',
	};

	let cs: CardState = {
		"card_map": new Map<number, Card>(),
		"fcards": [],
		"rm_stack": []
	}

	// all deck children of provided entry 
	let deck_names: string[] = [];

	async function getDecks() {
		let entryChildren: EntryChildren = await invoke(
			'read_decks', 
			{ "entry": $page.params.entry }
			);
		
		// extract deck names that are children of file system entry
		deck_names = entryChildren.deck_names;
		panel.selected_deck = deck_names[0];
		
		// load cards into frontend state, `panel.fcards` and `card_map`
		const cards = entryChildren.cards;
		for (let card of cards) {
			cs.card_map.set(card.fcard.id, card);
			cs.fcards.push(card.fcard);
		}

	} 
	getDecks();

	/*
	 * Button functionality: creating cards, multi-card creation, filtering gallery
	 */

	// number of cards created
	let numCreated = 0;
	async function createCard() {
		// don't save if either field is empty
		if (((panel.front === '' || panel.back === '') && !panel.making_multi)
			|| (panel.textfield === '' && panel.making_multi))
			return;
		
		if (panel.making_multi) {
			createCardTextfield()
			return;
		}

		// append to cards
		const front = panel.front;
		const back = panel.back;

		const id: number = await invoke(
			"calculate_hash", 
			{"deckName": panel.selected_deck, "front": front, "back": back }
			);
		const new_card: Card = {
			"fcard": {
				"id": id, 
				"front": front, 
				"back": back, 
				"deck_name": panel.selected_deck,
			},
			"md": {
				"last_review": "None",
				"box_pos": 0, 
				"is_created": true
			}
		};

		// add card to cards map and display if contains prompt
		cs.card_map.set(new_card.fcard.id, new_card);
		if (get_is_displayed(new_card.fcard)) {
			cs.fcards.splice(0, 0, new_card.fcard);
			cs.fcards = cs.fcards;
		}

		// cleanup panel fields
		panel.front = '';
		panel.back = '';

		// save all cards every fourth card made
		if (numCreated > 0 && numCreated % 4 == 0) 
			saveDecks()
		numCreated += 1;
	}

	async function createCardTextfield() {
		let fieldPairs: Array<FieldPair> = await invoke(
			'parse_textfield', 
			{ "textfield": panel.textfield }
		);
		panel.making_multi = false;

		let front_temp = panel.front;
		let back_temp = panel.back;

		for (const pair of fieldPairs) {
			panel.front = pair.front; 
			panel.back = pair.back; 
			createCard() 
		}
		
		panel.front = front_temp;
		panel.back = back_temp;
		panel.making_multi = true;
		panel.textfield = '';
		return;

	}

	function toggleMulti() {
		panel.display_multi = !panel.display_multi;
		panel.making_multi = !panel.making_multi;
	}

	function get_is_displayed(fcard: FrontendCard): boolean {
		if (fcard.front.includes(panel.prompt) ||
			fcard.back.includes(panel.prompt)) {
				return true;
			}
		return false;

	}
	
	async function filterCards() {	
		// save edited frontend cards to card_map before erasing fcards
		for (let fcard of cs.fcards) {
			let new_card = cs.card_map.get(fcard.id)!;
			new_card.fcard = fcard;
			cs.card_map.set(fcard.id, new_card);
		}

		cs.fcards = [];
		for (let [id, card] of cs.card_map) {
			if (get_is_displayed(card.fcard))
				cs.fcards.push(card.fcard);
		}

		cs.fcards = cs.fcards;
	}


	function deleteCard(card: FrontendCard) {
		// delete from gallery display
		const idx = cs.fcards.indexOf(card);
		if (idx > -1)
			cs.fcards.splice(idx, 1);

		// re-render gallery
		cs.fcards = cs.fcards;

		// delete in card state 
		let rmd_card = cs.card_map.get(card.id)!;
		// rmd_card does not exist if card with same id already deleted
		if (!rmd_card) 
			return;

		// push to rm_stack in case of undo 
		cs.rm_stack.push(rmd_card);
		// (note: delete unable to be undo'd if there was dup id removed)
		cs.card_map.delete(card.id);
	}

	function undoDelete() {
		let new_card = cs.rm_stack.pop();
		
		// do nothing if rm_stack is empty
		if (!new_card)
			return;

		// add card to cards map and display if contains prompt
		cs.card_map.set(new_card.fcard.id, new_card);
		if (get_is_displayed(new_card.fcard)) {
			cs.fcards.splice(0, 0, new_card.fcard);
			cs.fcards = cs.fcards;
		}

	}

	/*
	 * Animate cards: drag-and-drop and crossfade
	 */

	const dragDuration = 300
	let draggingCard: FrontendCard | undefined;
	let animatingCards = new Set()

	// swaps draggingCard with card
	function swapWith(card: FrontendCard) {
		if (draggingCard === card || animatingCards.has(card)) return
		animatingCards.add(card)
		setTimeout(() => animatingCards.delete(card), dragDuration)
		const cardAIndex = cs.fcards.indexOf(draggingCard!)
		const cardBIndex = cs.fcards.indexOf(card)
		cs.fcards[cardAIndex] = card
		cs.fcards[cardBIndex] = draggingCard!
	}

	/**
	 * Write changes to file system
	 */

	// save cards; called on exit (press 'home') or every four cards
	async function saveDecks() {

		// save changes of edited cards
		for (let fcard of cs.fcards) {
			let new_card = cs.card_map.get(fcard.id);
			if (!new_card)
				continue;

			new_card.fcard = fcard;
			cs.card_map.set(fcard.id, new_card);
		}

		const cards = Array.from(cs.card_map.values());
		await invoke('write_decks', { "cards": cards });

	}

	
</script>
<a href="/"><button on:click={saveDecks}>Home</button></a>


<div class="panel">
	<!-- choose deck name; `selected_deck_name` by default -->
	<select bind:value={panel.selected_deck} name="deck_menu" id="deck_menu">
		{#each deck_names as deck_name}
			<option value={deck_name}> {deck_name} </option>
		{/each}
	</select>

	{#if !panel.display_multi}
	<!-- show center card field -->
	<div>
		<div class="front">
			<textarea class="panel_text" bind:value={panel.front} />
		</div>
			
		<div class="back">
				<textarea class="panel_text" bind:value={panel.back} />
		</div>
	</div>
	{:else}
	<div>
		<textarea id="multi_field" bind:value={panel.textfield}></textarea>
	</div>

	{/if}
		
		
	<!-- sumbit card or change to multi-card -->
	<div class="submit_bar">
		<button on:click={createCard}> |-> </button>
		<button on:click={toggleMulti}> >> </button>
	</div>

	<div class="lookup_bar">
		<Search bind:prompt={panel.prompt} on:input={filterCards} />
		<button on:click={() => undoDelete()}>undo</button>
	</div>

</div>

<div class="container">
	<!-- Note: the keyed index must be (card) for the animation to work -->
	{#each cs.fcards as card (card)}
		<div
			animate:flip={{ duration: dragDuration }}
			class="card"
			draggable="true"
			on:dragstart={() => draggingCard = card}
			on:dragend={() => draggingCard = undefined}
			on:dragenter={() => swapWith(card)}
			on:dragover|preventDefault
		>
			<!-- bar above card -->
		 	<div class="card_id">
				<!-- change deck of card -->
				<select bind:value={card.deck_name} name="deck_menu" id="deck_menu">
					{#each deck_names as deck_name}
						<option value={deck_name}> {deck_name} </option>
					{/each}
				</select>
				<button on:click={() => deleteCard(card)}>delete</button>
			</div>

			<!-- card fields -->
			<div class="front">
				<textarea bind:value={card.front} />
			</div>
			
			<div class="back">
				<textarea bind:value={card.back} />
			</div>
			
		</div>
	{/each}
</div>

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
	
	/* .panel_card {
		display: flex;
 		flex-direction:column;
		padding: 8px;
		justify-content: center;
		align-items: center;
		
		width: 100%;
		height: 100%;
		font-size: 1.5rem;
		
	} */
	
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

	/* #multi_field { */
		/* display: flex;
		flex-direction:column;
		padding: 8px;
		justify-content: center;
		align-items: center;
		
		width: 100%;
		height: 100%;
		font-size: 16px; */

		
	</style>