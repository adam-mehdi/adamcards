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

		entryChildren.deck_names = entryChildren.deck_names.map(s => s.replace("~~", "/"))
		deck_names = entryChildren.deck_names;
		panel.selected_deck = deck_names[0];
		
		// load cards into frontend state, `panel.fcards` and `card_map`
		const cards = entryChildren.cards;
		for (let card of cards) {
			cs.card_map.set(card.fcard.id, card);
			cs.fcards.push(card.fcard);
		}

		// render gallery in DOM
		cs.fcards = cs.fcards;

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

	let multiInput: HTMLElement;
	let firstInput: HTMLElement;

	function toggleMulti() {
		panel.display_multi = !panel.display_multi;
		panel.making_multi = !panel.making_multi;
	}

	function get_is_displayed(fcard: FrontendCard): boolean {
		let prompt = panel.prompt.toLowerCase();
		if (fcard.front.toLowerCase().includes(prompt) ||
			fcard.back.toLowerCase().includes(prompt)) {
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
			fcard.deck_name = fcard.deck_name.replace("/", "~~");
			new_card.fcard = fcard;
			cs.card_map.set(fcard.id, new_card);
		}

		const cards = Array.from(cs.card_map.values());
		await invoke('write_decks', { "cards": cards });

	}

	
</script>

<a class="home-button" href="/"><button on:click={saveDecks}>Home</button></a>
	<!-- choose deck name; `selected_deck_name` by default -->


<div class="panel">
	{#if !panel.display_multi}
	<!-- show center card fields -->


		<div class="card">
			<div class="create-card-front front">                               
                <textarea          
					class="panel_text"                                                    
                    id="upper-field"                                            
                    bind:this={firstInput}                                                                         
                    bind:value={panel.front}                                    
                    autofocus                                                   
                />                                                              
            </div>          

			<!-- rule separating front and back fields -->
			<div class="card-hr" />      

			<div class="create-card-back back">                                 
                <textarea 
					class="panel_text" 
					bind:value={panel.back} 
				/>         
            </div>  


		<div class="card-create-buttons">                                   
			<button on:click={createCard}>Create {!panel.display_multi ? "Card" : "Cards"}</button>              
																			
			<button on:click={toggleMulti}>{!panel.display_multi ? `Multi Editor` : `Single Editor`}</button>

			<!-- change class of whether deleted based on whether there are deleted cards -->
			<button class={cs.rm_stack.length == 0 ? "hidden" : ""} on:click={undoDelete}>Undo Delete</button>

			<!-- bar below card -->
			<select class="panel-deck-menu" id="panel-deck-menu" bind:value={panel.selected_deck}>
				{#each deck_names as deck_name}
					<option value={deck_name}> {deck_name} </option>
				{/each}
			</select>

		</div>       
	</div>                

	{:else}
	<div class="card multi">                                                
		<textarea bind:this={multiInput} bind:value={panel.textfield} autofocus />
		<div class="card-create-buttons">                                   
			<button on:click={createCard}>Create {!panel.display_multi ? "Card" : "Cards"}</button>              
																			
			<button on:click={toggleMulti}>{!panel.display_multi ? `Multi Editor` : `Single Editor`}</button>
		</div>                                                              
	</div>           

	{/if}
		
	<!-- lookup prompt -->                                
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
	<div class="card-container">   
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
				<select class="card-deck-menu" id="card-deck-menu" bind:value={card.deck_name}>
					{#each deck_names as deck_name}
						<option value={deck_name}> {deck_name} </option>
					{/each}
				</select>

				<!-- <p class=deck-menu>{card.deck_name.replace("~~", "/")}</p> -->

				<!-- delete button at top right -->
				<span 
					class="remove" 
					on:click={() => deleteCard(card)} 
					on:keydown={() => deleteCard(card)}
					> 
					✕
				</span>

				<!-- card fields -->
				<div class="front card-input">
					<textarea bind:value={card.front} />
				</div>
				<div class="card-hr" />
				
				<div class="back card-input">
					<textarea bind:value={card.back} />
				</div>
				
			</div>
		{/each}
	</div>
</div>

<style>
	.multi textarea {                                                           
        width: 90vw;                                                            
        max-width: 500px;                                                       
        height: 200px;                                                          
    }                                                                           
                                                                                
    button {                                                                    
        border: none;                                                           
        height: 32px;                                                            
        border-radius: 0.3em;                                                   
		background-color: #e1dfdd;; 
		color: #1f1f1f;
		cursor: pointer;
    }                                                                           
                                                                                
    .create-card-front textarea {                                               
        max-width: 500px;                                                       
        width: 95vw;                                                            
    }                                                                           
                                                                                
    .create-card-back textarea {                                                
        max-width: 500px;                                                       
        width: 95vw;                                                            
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
        font-family:  Cambria, Cochin, Georgia, Times, 'Times New Roman', serif;                                                
		font-size: 15px;
		line-height: 140%;
        width: 256px;                                                           
        height: 64px;                                                           
        resize: None;                                                           
        border: none;                                                           
        border-radius: 0.8em;                                                   
        padding: 0.8em;                                                         
		outline-color: #B9D9EB;         
                           
    }                                                                           
                                                                                
    .back textarea {                                                            
        border-radius: 0.2em 0.2em 0.8em 0.8em;                                 
    }                                                                           
                                                                                
    .front textarea {                                                           
        border-radius: 0.8em 0.8em 0.2em 0.2em;                                 
    }                                                                           

	/* add image support and submit bar at bottom    */                         
    .panel {                                                                    
        margin-top: 100px;                                                      
        /* height: 38vh; */                                                     
        display: flex;                                                          
        flex-direction: column;                                                 
        justify-content: center;                                                
        align-items: center;                                                    
    }                                                                           
                                                                                
                                                                                
    .home-button {                                                              
        /* position: fixed;                                                         */
        /* left: 1em;                                                               */
        /* bottom: 1em;                                                             */
		cursor: pointer;
    }                                                                           
                                                                                
	.card-create-buttons button {                                               
        margin: 1em;      
		cursor: pointer;                                                      
    }                                                                           
                                                                                
    #search-input {                                                             
		outline-color: #B9D9EB;                                    
        font-size: 1em;                                                         
		text-align: center;
    }                                                                           
                                                                                
    .lookup-bar-card {                                                          
        margin: 1em;
		width: 30%;
		height: 20px;
    }         
	
	/* .deck-menu {
		opacity: .3;
		position: relative;
		top: 8px;
		font-size: 10px;

	} */


	@media (hover: hover) {
		.remove {
			visibility: hidden;
		}
		.card:hover .remove {
			visibility: visible;
			cursor: pointer;
			position: relative;
			left: 260px;
			bottom: 20px;
			color:#e1dfdd;
			user-select: none;
		}

		.card-deck-menu {
			visibility: hidden;
		}

		.card:hover .card-deck-menu {
			visibility: visible;
			position: relative;
			width: 150px;
			height: 16px;
			left: 55px;

			border-style: none;
			border-radius: 10px !important;                                         
			font-family: 'Courier New', Courier, monospace;
			opacity: .6;
			cursor: pointer;
			                            
		}


	}
	
	.panel-deck-menu {
		visibility: visible;
		position: relative;
		width: 150px;
		height: 28;
		/* left: 55px; */

		border-style: none;
		border-radius: 10px !important;                                         
		font-family: 'Courier New', Courier, monospace;
		opacity: .6;
		cursor: pointer;
									
	}

	.hidden {
		visibility: hidden;
	}

</style>