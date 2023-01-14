<script lang="ts">
	import { flip } from 'svelte/animate';
	import Search from '$lib/SearchBarFilter.svelte';
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';

	// load cards from deck
	interface Card {
		id: number;
		front: string;
		back: string;
		last_review: string;
		box_pos: number;
		deck_name: string;
		is_created: boolean;
	}

	interface CenterPanel {
		front: string;
		back: string;
		prompt: string;
		selected_deck: string;
		display_multi: boolean;
		making_multi: boolean;
		textfield: string;
	}

	interface FieldPair {
		front: string;
		back: string;
	}

	// all decks that are children of the provided file system entry
	interface EntryChildren {
		cards: Card[];
		deck_names: string[];
	}

	/*
	 * Initialize state
	 */

	// init panel
	let panel: CenterPanel = {
		front: '',
		back: '',
		prompt: '',
		selected_deck: '',
		display_multi: false,
		making_multi: false,
		textfield: ''
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
		await invoke('write_decks', { cards: cards, numCreated: numCreated });
	}

	async function createCard() {
		// don't save if either field is empty
		if (panel.display_multi) {
			multiInput.focus();
		} else {
			firstInput.focus();
		}
		if (
			((panel.front === '' || panel.back === '') && !panel.making_multi) ||
			(panel.textfield === '' && panel.making_multi)
		)
			return;

		if (panel.making_multi) {
			let fieldPairs: Array<FieldPair> = await invoke('parse_textfield', {
				textfield: panel.textfield
			});
			panel.making_multi = false;

			let front_temp = panel.front;
			let back_temp = panel.back;

			for (const pair of fieldPairs) {
				panel.front = pair.front;
				panel.back = pair.back;
				createCard();
			}

			panel.front = front_temp;
			panel.back = back_temp;
			panel.making_multi = true;
			panel.textfield = '';
			return;
		}

		let front = panel.front;
		let back = panel.back;

		// append to cards
		const id: number = await invoke('calculate_hash', {
			deckName: panel.selected_deck,
			front: front,
			back: back
		});

		const new_card: Card = {
			id: id,
			front: front,
			back: back,
			last_review: '0',
			box_pos: 0,
			deck_name: panel.selected_deck,
			is_created: true
		};
		console.log(new_card);

		// TODO: crossfade animation and prepend
		cards.splice(0, 0, new_card);
		cards = cards;
		numCreated += 1;

		panel.front = '';
		panel.back = '';
		// save  all cards every fourth card made
		if (numCreated > 0 && numCreated % 4 == 0) {
			saveDecks();
		}

		// Focus the the form to create the next card.
		// document.getElementById('upper-input')?.focus();
		// console.log('Ran focus');
	}

	let multiInput: HTMLElement;
	let firstInput: HTMLElement;

	function toggleMulti() {
		panel.display_multi = !panel.display_multi;
		panel.making_multi = !panel.making_multi;
	}

	const filterCards = () => {
		// return array of cards that match search term
	};

	function deleteCard(card: Card) {
		const index = cards.indexOf(card);
		if (index > -1) {
			cards.splice(index, 1);
		}
		cards = cards;
	}

	/*
	 * Animate cards: drag-and-drop and crossfade
	 */

	const dragDuration = 300;
	// let cards: number[] = Array(20).fill(1).map((_, i) => i + 1)
	let draggingCard: Card | undefined;
	let animatingCards = new Set();

	function swapWith(card: Card) {
		if (draggingCard === card || animatingCards.has(card)) return;
		animatingCards.add(card);
		setTimeout(() => animatingCards.delete(card), dragDuration);
		const cardAIndex = cards.indexOf(draggingCard!);
		const cardBIndex = cards.indexOf(card);
		cards[cardAIndex] = card;
		cards[cardBIndex] = draggingCard!;
	}
</script>

<a class="home-button" href="/"><button on:click={saveDecks}>Home</button></a>
<div class="editing-deck-menu">
	Editing:
	<select class="deck-menu" name="deck_menu" id="deck_menu">
		{#each deck_names as deck_name}
			<option value={deck_name}> {deck_name} </option>
		{/each}
	</select>
</div>
<div class="panel">
	<!-- choose deck name; `selected_deck_name` by default -->
	{#if !panel.display_multi}
		<!-- show center card field -->
		<div class="card">
			<div class="create-card-front front">
				<textarea
					id="upper-field"
					bind:this={firstInput}
					class="panel_text"
					bind:value={panel.front}
					autofocus
				/>
			</div>
			<div class="card-hr" />

			<div class="create-card-back back">
				<textarea class="panel_text" bind:value={panel.back} />
			</div>

			<div class="card-create-buttons">
				<button on:click={createCard}>Create Card</button>

				<button on:click={toggleMulti}>{!panel.display_multi ? `Multi` : `Single`}</button>
			</div>
		</div>
	{:else}
		<div class="card multi">
			<textarea bind:this={multiInput} bind:value={panel.textfield} autofocus />
			<div class="card-create-buttons">
				<button on:click={createCard}>Create Card</button>

				<button on:click={toggleMulti}>{!panel.display_multi ? `Multi` : `Single`}</button>
			</div>
		</div>
	{/if}

	<!-- sumbit card or change to multi-card -->
	<div class="card lookup-bar-card">
		<input
			type="text"
			id="search-input"
			placeholder="filter cards"
			autocomplete="off"
			bind:value={panel.prompt}
			on:input={filterCards}
		/>
	</div>
</div>

<div class="card-container-container">
	{#if cards.length > 0}
		<div class="card-container">
			{#each cards as card (card)}
				<div
					animate:flip={{ duration: dragDuration }}
					class="card"
					draggable="true"
					on:dragstart={() => (draggingCard = card)}
					on:dragend={() => (draggingCard = undefined)}
					on:dragenter={() => swapWith(card)}
					on:dragover|preventDefault
				>
					<div class="front card-input">
						<textarea bind:value={card.front} />
					</div>
					<div class="card-hr" />

					<div class="back card-input">
						<textarea bind:value={card.back} />
					</div>

					<div class="card-info">
						<span class="card-deck-name">{card.deck_name}</span>
						<button on:click={() => deleteCard(card)}>delete</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.multi textarea {
		width: 90vw;
		max-width: 500px;
		height: 200px;
	}

	button {
		border: none;
		height: 2em;
		border-radius: 0.3em;
	}

	.create-card-front textarea {
		max-width: 500px;
		width: 95vw;
	}

	.create-card-back textarea {
		max-width: 500px;
		width: 95vw;
	}

	.editing-deck-menu {
		font-style: italic;
		position: fixed;
		top: 1em;
		height: 1em;
	}

	.card-container-container {
		margin-top: 2em;
	}
	.card-container {
		width: 100%;
		height: fit-content;
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, max-content));
		grid-gap: 16px;
		justify-content: center;
		padding: initial;
	}

	.card {
		width: 300px;
		display: flex;
		flex-direction: column;
		padding: 8px;
		justify-content: center;
		align-items: space-between;

		width: min-content;
		height: min-content;
		/* font-size: 1.5rem; */
		border-radius: 1em;

		border: 0px solid #e1dfdd;
		box-shadow: 0 10px 20px -8px rgba(197, 214, 214);
		transition: all 0.3s cubic-bezier(0, 0, 0.5, 1);
		border-radius: 10px !important;
		background-color: white;
	}

	.card-hr {
		border-top: 1px solid #e1dfdd;
	}

	.card-input {
		border: none;
		border-color: none;
		border-radius: 1em;
	}

	textarea {
		font-family: sans-serif;
		width: 256px;
		height: 64px;
		resize: None;
		border: none;
		border-radius: 0.8em;
		padding: 0.8em;
		outline-color: #9bcbeb;
	}
	textarea:focus {
	}

	.back textarea {
		border-radius: 0.2em 0.2em 0.8em 0.8em;
	}

	.front textarea {
		border-radius: 0.8em 0.8em 0.2em 0.2em;
	}

	/* add image support and submit bar at bottom	 */
	.panel {
		margin-top: 100px;
		/* height: 38vh; */
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
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

	/* .panel_text {
		width: 312px;
		height: 86px;
		border-radius: 16px;
		resize: None;
		font-size: px;
	} */

	.lookup-bar {
		width: 200px;
	}

	button {
		height: 32px;
		/* font-size: 12px; */
	}

	.home-button {
		position: fixed;
		left: 1em;
		bottom: 1em;
	}

	.card-info {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
	}

	.card-create-buttons {
		display: flex;
		justify-content: center;
		align-items: center;
		width: 100%;
	}

	.card-create-buttons button {
		margin: 1em;
	}
	.card-deck-name {
		margin-left: 0.2em;
	}

	#search-input {
		outline-color: #9bcbeb;
		font-size: 1em;
	}

	.lookup-bar-card {
		margin: 2em;
	}
</style>
