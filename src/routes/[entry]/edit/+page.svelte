
<script lang="ts">
	import { flip } from 'svelte/animate'
	import { page, updated } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';
	import configStore from '$lib/stores/configStore'
	import Hint from 'svelte-hint';
	import {clickOutside} from '$lib/actions/click_outside.js';

	import { onMount, onDestroy } from 'svelte'
	import TextfieldEditor from '$lib/TextfieldEditor.svelte'

	


	

	// contains state for the central panel on which cards are created
	interface CenterPanel {
		front: string,
		back: string,
		expl: string,
		prompt: string,
		selected_deck: string,
		display_all: boolean,
		display_textfield: boolean,
		textfield: string,
	}


	interface DeckContents {
    	deck_id: number,
    	deck_name: string,
    	cards: Card[]
	}

	interface Card {
		id: number,
		front: string,
		back: string
	}

	interface CardDisplay {
		is_visible: boolean,
		deck_name: string,
		show_expl: boolean,
		card: Card
	}


	/*
	 * Initialize state
	 */
	let isDarkMode: boolean = $configStore.is_dark_mode;
	const deadlineId = parseInt($page.params.entry);

	// init panel
	let panel: CenterPanel = {
		"front": '',
		"back": '',
		"expl": '',
		"prompt": '',
		"display_textfield": $configStore.is_textfield,
		"textfield": '',
		"display_all": false,
		"selected_deck": '',
	};

	// all deck children of provided entry 
	let deadlineContents: DeckContents[]
	let card_gallery: CardDisplay[] = [];
	let rm_stack: CardDisplay[] = []


	async function getDeadlineContents() {

		deadlineContents = await invoke('read_deadline_contents', { deadlineId });
		if (deadlineContents.length == 0) return
		panel.selected_deck = deadlineContents[0].deck_name.toString();

		for (let deckContents of deadlineContents) {
			for (let card of deckContents.cards) {
				let card_display: CardDisplay = { 
					"is_visible": false, 
					"deck_name": deckContents.deck_name,
					"show_expl": true,
					card 
				};
				card_gallery.push(card_display);
			}
		}
		filterCards()


		if (deadlineContents.length == 0) 
			createDeckTrayOpen = true;
	} 
	getDeadlineContents();


	interface NewCard {
		front: string,
		back: string
	}

	let clearEditorToggle = false;
	async function createCardsBackend(newCards: NewCard[]) {
		// don't save if either field is empty
		const isFrontBack = !(panel.front === '' || panel.back === '' && !panel.display_textfield)
		const isTextfield = !(panel.textfield == '' && panel.display_textfield)

		if (!isFrontBack && !isTextfield)
			return;

		
		// append to cards
		let deckName = panel.selected_deck;

		let deckContents = deadlineContents.filter((x) => x.deck_name == deckName)[0];
		let deckNewContents = {
			"deck_name": deckName,
			"deck_id": deckContents.deck_id,
			"cards": newCards
		}
		let ids: number[] = await invoke("create_cards", { deadlineId, deckNewContents });
		for (const [idx, new_card] of deckNewContents.cards.entries()) {
			let card = {
				"id": ids[idx],
				"front": new_card.front,
				"back": new_card.back
			}
			deckContents.cards.push(card);

			let cardDisplay = { 
				"is_visible": false, 
				"deck_name": deckContents.deck_name,
				"card": card,
				"show_expl": true
			}
			cardDisplay.is_visible = get_is_displayed(cardDisplay)
			card_gallery.splice(0, 0, cardDisplay)
			// getExplanation(cardDisplay)
		}
		card_gallery = card_gallery
	}

	async function createCardFrontBack() {
		if (await checkDeadlinePast()) return
		let card: NewCard = { "front": panel.front, "back": panel.back }
		createCardsBackend([card])
		panel.front = ''
		panel.back = ''
		clearEditorToggle = !clearEditorToggle;

	}

	async function createCardTextfield() {
		if (await checkDeadlinePast()) return
		if (panel.textfield == "")
			return;

		let pairs = parse_textfield();
		
		createCardsBackend(pairs) ;

		panel.textfield = '';
		clearEditorToggle = !clearEditorToggle;
	}

	function parse_textfield(): NewCard[] {
		let txt = panel.textfield;
		txt = txt.replaceAll("•", "");
		let newCards: NewCard[] = [];

		let results = txt.match(/<(div|ul|li)>.*?».*?<\/(div|ul|li)>/g);

		if (results) {
			for (let result of results) {
				result = result.substring(result.indexOf(">")+1, result.lastIndexOf("<"));
				let style_match = result.match(/(style=").*?"/g);
				if (style_match != null)
					result = result.replaceAll(style_match[0], "");
				
				
				for (let line of result.split("<br>")) {
					let card = line.split("»");
					if (card && card.length == 2) {
						let front = `<div>${card[0].trim()}</div>`;
						let back = `<div>${card[1].trim()}</div>`;
						newCards.push({ front, back });
					}
				}
			}
		}


		return newCards;
	}


	function toggleTextfield() {
		panel.display_textfield = !panel.display_textfield;
	}

	$: configStore.update((config) => {
		config.is_textfield = panel.display_textfield;
		return config;
	})

	onDestroy(async () => {
		invoke("write_text_field", { "isTextField": panel.display_textfield });
	});


	// === filter bar ===
	function get_is_displayed(card: CardDisplay): boolean {
		let prompt = panel.prompt.toLowerCase();
		const inPrompt = (x: string) => x.toLowerCase().includes(prompt);
		if (inPrompt(card.card.front) || inPrompt(card.card.back)) {
				if (panel.display_all || card.deck_name == panel.selected_deck)
					return true;
			}
		return false;

	}
	
	function filterCards() {	
		for (let card of card_gallery) {
			if (get_is_displayed(card))
				card.is_visible = true;
			else
				card.is_visible = false;
		}

		card_gallery = card_gallery;
	}
	$: panel.selected_deck, filterCards()
	// ===


	/// === gallery ===
	function deleteCard(card: CardDisplay) {
		// delete from gallery display
		let idx = card_gallery.indexOf(card);
		if (idx < -1) return
		rm_stack.push(card_gallery[idx]);
		card_gallery.splice(idx, 1);
		
		let deckContents: DeckContents = deadlineContents.filter((x: DeckContents) => x.deck_name == card.deck_name)[0]
		idx = deckContents.cards.indexOf(card.card)
		deckContents.cards.splice(idx, 1);

		invoke("delete_card", {"cardId": rm_stack[rm_stack.length - 1].card.id})

		// re-render gallery
		card_gallery = card_gallery;
		rm_stack = rm_stack;
	}

	function undoDelete() {
		if (rm_stack.length == 0) return
		let new_card = rm_stack.pop()!;
		
		// add card to cards map and display if contains prompt
		card_gallery.splice(0, 0, new_card)
		let deckContents: DeckContents = deadlineContents.filter((x: DeckContents) => x.deck_name == new_card.deck_name)[0]
		deckContents.cards.push(new_card.card)

		let deckNewContents = {
			"deck_name": deckContents.deck_name,
			"deck_id": deckContents.deck_id,
			"cards": [new_card.card]
		}
		invoke("create_cards", { deadlineId, deckNewContents });
		card_gallery = card_gallery;

	}


	function updateCard(card: Card) {
		invoke("update_card", { card })
	}

	/*
	 * Animate cards: drag-and-drop and crossfade
	 */

	const dragDuration = 150
	let draggingCard: CardDisplay | undefined;
	let animatingCards = new Set()

	// swaps draggingCard with card
	function swapWith(card: CardDisplay) {
		if (draggingCard === card || animatingCards.has(card)) return
		animatingCards.add(card)
		setTimeout(() => animatingCards.delete(card), dragDuration)
		const cardAIndex = card_gallery.indexOf(draggingCard!)
		const cardBIndex = card_gallery.indexOf(card)
		card_gallery[cardAIndex] = card
		card_gallery[cardBIndex] = draggingCard!
	}
	// ===


	let createDeckTrayOpen = false;
	function toggleCreateDeckTray() {
		createDeckTrayOpen = !createDeckTrayOpen;
		newName = ""
	}

	let newName = "";
	let deadline_is_complete: boolean;
	async function checkDeadlinePast(): Promise<boolean> {
		let deadline_tuple: [String, boolean] | null = await invoke("get_deadline_date", { deadlineId });
		if (!deadline_tuple) {
			return false;
		}
		deadline_is_complete = deadline_tuple[1];
		if (deadline_is_complete) {
			createDeckTrayOpen = false
			clearEditorToggle = !clearEditorToggle
		}
			
		return deadline_is_complete;
	}

	async function createDeck() {
		// entry_deadline_date, deadline_complete
		if (await checkDeadlinePast()) return

		
		if (newName == "" || deadlineContents.some((x) => x.deck_name === newName)) return

		let md = {
			entry_type: "deck",
			deadline_date: null,
			study_intensity: null
		}
		newName = newName.slice(0, 29);
		invoke("create_entry", { "entryName": newName, "parentId": deadlineId, md});
		card_gallery = [] // avoid duplicate card in gallery
		getDeadlineContents()
		toggleCreateDeckTray()
	}
	function focus(el: any){
    	el.focus()
  	}

	function toggleShowExplanation(card: CardDisplay) {
		card.show_expl = !card.show_expl;
		card_gallery = card_gallery
	}


</script>

	
<div class="{isDarkMode ? "dark" : ""}">
	<div class="min-h-screen h-full bg-offwhite dark:bg-offblack ">
		<div class="">
		
		<div class="flex justify-start space-x-5 mx-10">
			
			<!-- home button -->
			<a href="/" id="home-button" class="outline-columbia  ring-columbia mt-1 focus:outline-none focus:ring duration-75 rounded-md">
				<div class="fled justify-evenly ring-columbia  focus:outline-none focus:ring duration-75 rounded-md">
					<svg 
						fill="highlight" class="fill-columbia float-left h-7 w-7 cursor-pointer outline-columbia focus:outline outline-4 outline-offset-2 " 
						viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path clip-rule="evenodd" fill-rule="evenodd" d="M9.293 2.293a1 1 0 011.414 0l7 7A1 1 0 0117 11h-1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-3a1 1 0 00-1-1H9a1 1 0 00-1 1v3a1 1 0 01-1 1H5a1 1 0 01-1-1v-6H3a1 1 0 01-.707-1.707l7-7z"></path>
					</svg>
				</div>
			</a>

			<div class="bg-offwhite z-50 dark:bg-offblack">
				{#if !panel.display_textfield}
					<!-- front-back -->
					<button 
						class="h-9 w-9 ring-columbia mt-1  focus:outline-none focus:ring duration-75 rounded-md"
						on:click={toggleTextfield}>
				<Hint placement="bottom" text="Enable textfield editor">
						<svg 
							class="fill-columbia rounded-lg bg-inherit cursor-pointer h-7 w-7 outline-columbia focus:outline outline-4 outline-offset-2 " 
							fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
							<path d="M5.127 3.502L5.25 3.5h9.5c.041 0 .082 0 .123.002A2.251 2.251 0 0012.75 2h-5.5a2.25 2.25 0 00-2.123 1.502zM1 10.25A2.25 2.25 0 013.25 8h13.5A2.25 2.25 0 0119 10.25v5.5A2.25 2.25 0 0116.75 18H3.25A2.25 2.25 0 011 15.75v-5.5zM3.25 6.5c-.04 0-.082 0-.123.002A2.25 2.25 0 015.25 5h9.5c.98 0 1.814.627 2.123 1.502a3.819 3.819 0 00-.123-.002H3.25z"></path>
						</svg>
				</Hint>
					</button>


				{:else}
					<!-- textfield -->
				<button on:click={toggleTextfield} class="h-9 w-9 z-50 dark:bg-offblack ring-columbia mt-1 focus:outline-none focus:ring duration-75 rounded-md">
					<Hint placement="bottom" text="Enable front/back editor">
							<svg 
								class="fill-columbia  bg-inherit cursor-pointer h-7 w-8 outline-columbia focus:outline outline-4 outline-offset-2 " 
								fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
								<path d="M5.25 3A2.25 2.25 0 003 5.25v9.5A2.25 2.25 0 005.25 17h9.5A2.25 2.25 0 0017 14.75v-9.5A2.25 2.25 0 0014.75 3h-9.5z"></path>
							</svg>
					</Hint>
				</button>
				{/if}
			</div>

			{#if !deadline_is_complete}
				<div class="bg-offwhite z-50 dark:bg-offblack" >
					<button on:click={toggleCreateDeckTray} class="h-9 w-9 z-50 dark:bg-offblack ring-columbia mt-1 focus:outline-none focus:ring duration-75 rounded-md">
						<Hint placement="bottom" text="Create deck">
							<svg 
								class="fill-columbia  bg-inherit cursor-pointer h-7 w-8 outline-columbia focus:outline outline-4 outline-offset-2 " 
								xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" >
								<path fill-rule="evenodd" d="M5.625 1.5H9a3.75 3.75 0 013.75 3.75v1.875c0 1.036.84 1.875 1.875 1.875H16.5a3.75 3.75 0 013.75 3.75v7.875c0 1.035-.84 1.875-1.875 1.875H5.625a1.875 1.875 0 01-1.875-1.875V3.375c0-1.036.84-1.875 1.875-1.875zM12.75 12a.75.75 0 00-1.5 0v2.25H9a.75.75 0 000 1.5h2.25V18a.75.75 0 001.5 0v-2.25H15a.75.75 0 000-1.5h-2.25V12z" clip-rule="evenodd" />
								<path d="M14.25 5.25a5.23 5.23 0 00-1.279-3.434 9.768 9.768 0 016.963 6.963A5.23 5.23 0 0016.5 7.5h-1.875a.375.375 0 01-.375-.375V5.25z" />
							</svg>
						</Hint>
					</button>
				</div>
			{/if}
			  

		</div>
		{#if createDeckTrayOpen}
			<form class="w-1/3 absolute left-40 py-1 z-50 bg-offwhite dark:bg-offblack dark:text-whitetext"
					on:submit={createDeck} use:clickOutside on:click_outside={toggleCreateDeckTray}>
					<!-- disregard the above warning -->
				<div class="border p-4 rounded-lg rounded-tl-sm border-columbia py-2 grid grid-rows-2 grid-cols-3 gap-2">
					<!-- name for Create -->
					<input type="text" use:focus placeholder="Enter Deck Name" bind:value={newName} class="h-8 col-span-3 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring-2 duration-75"/>
					<button type="submit" class="h-8 col-span-2 text-sm hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring-2 duration-75">
						Create Deck	
					</button>
					<button type="button" on:click={ toggleCreateDeckTray } 
						class="h-8 col-span-1 hover:bg-columbia border-2 border-columbia dark:bg-offblack text-sm dark:hover:bg-columbia-dark rounded-lg block dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring-2 duration-75">
						Cancel
					</button>
				</div>
			</form>


		{/if}
		
	
	<div class="h-5"></div>

	 <!-- central panel -->
	<div class="mt-4 h-full flex flex-col justify-center items-center">
		<!-- card fields -->
		<div class="card flex flex-col h-full rounded-lg text-blacktext w-full sm:w-[600px] md:w-[650px] lg:w-[700px] dark:text-whitetext">

			{#key clearEditorToggle}
			{#if !panel.display_textfield}
				
				<!-- front field -->
				<div class="h-full max-h-80 mx-2 mb-1 border-l rounded-md border-columbia" >         
					<div class="h-full p-1 rounded-lg ">
						<TextfieldEditor bind:content={panel.front} autofocus={true}/>
					</div>
				</div>          

				<!-- back field -->
				<div class="h-full max-h-80 mx-2 border-l rounded-md mb-1 border-columbia" >         
					<div class="h-full p-1 rounded-lg ">
						<TextfieldEditor bind:content={panel.back} />
					</div>
				</div>          
					

			{:else}
				<div class="h-full m-3 mb-2 text-inherit" >         
					<div class="h-full p-1 rounded-lg border-l border-columbia">
						<TextfieldEditor bind:content={panel.textfield} is_textfield={true}/>
					</div>
				</div>
			{/if}
			{/key}
			
			<!-- make this one bar -->
			<div class="flex items-center justify-center top-[450px]">     
			{#if deadlineContents && deadlineContents.length > 0 && !deadline_is_complete}

				<!-- Create -->
				<button 
					on:click={panel.display_textfield ? createCardTextfield : createCardFrontBack} 
					class="h-8 w-1/4 relative z-50 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all border-y border-l border-gray-200 rounded-bl-lg cursor-pointer group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75">
					<svg class="fill-columbia stroke-columbia flex-none h-6 w-6 outline-columbia focus:outline outline-4 outline-offset-2 "
						stroke="currentColor" stroke-width="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15"></path>
					</svg>
				</button>


				<!-- select deck -->
				<div style="text-align-last:center;" class="border-y px-3 py-1 w-1/2 h-8 relative z-30 outline-columbia focus:outline outline-4 outline-offset-2 overflow-hidden font-bold text-gray-500 transition-all duration-200 border-gray-200 cursor-pointer group ease border-columbia  bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white">
					<select bind:value={panel.selected_deck}
						class="text-columbia relative appearance-none w-full h-full outline-columbia focus:outline outline-4 outline-offset-2 justify-center z-30 inline-flex overflow-hidden font-bold text-gray-500 border-gray-200 cursor-pointer group ease border-columbia bg-gradient-to-b from-offwhite dark:from-offblack dark:to-offblack dark:border-x dark:rounded-none to-gray-50 hover:from-gray-50 hover:to-white active:to-white transition-all duration-100"
					>
					 {#if deadlineContents}
						{#each deadlineContents as deckContents}
							<option value={deckContents.deck_name}>
								{deckContents.deck_name}
							</option>
						{/each}	
					{/if}
					</select>
				</div>



				<!-- Undo button -->
				{#if rm_stack.length > 0}
					<button
						on:click={undoDelete}
						class="h-8 w-1/4 relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all border-y border-r border-gray-200 rounded-br-lg cursor-pointer group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75"
						>
						<!-- Undo -->
						<svg class="fill-columbia flex-none h-7 w-7 cursor-pointer outline-columbia focus:outline outline-4 outline-offset-2 "
							fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
							<path clip-rule="evenodd" fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"></path>
						</svg>
					</button>
				{:else}
					<button 
						class="h-8 w-1/4 relative z-30 text-platinum inline-flex items-center justify-center px-8 py-3 pt-4 overflow-hidden font-bold text-gray-500 transition-all border-y border-r border-gray-200 rounded-br-lg cursor-default group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75"
					>
						<svg class="fill-columbia opacity-50 flex-none h-8 w-8 outline-columbia focus:outline outline-4 outline-offset-2"
							viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
							<path clip-rule="evenodd" fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"></path>
						</svg>
					</button>
				{/if}

			{:else if deadline_is_complete}
				<div class="h-7 w-full font-mono relative z-30 text-platinum inline-flex items-center justify-center px-8 py-4 overflow-hidden font-bold text-gray-500 transition-all border border-gray-200 rounded-b-lg cursor-not-allowed group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75">
					deadline past: return home and reset it
				</div>
			{:else}
			<button on:click={toggleCreateDeckTray} class="w-full">
				<div class="h-8 w-full font-mono relative z-30 text-platinum inline-flex items-center justify-center px-8 py-4 overflow-hidden font-bold text-gray-500 transition-all border border-gray-200 rounded-b-lg group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75">
						<div class="mr-2">create deck</div>
						<svg 
							class="fill-columbia  bg-inherit cursor-pointer h-6 w-8 outline-columbia focus:outline outline-4 outline-offset-2 " 
							xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" >
							<path fill-rule="evenodd" d="M5.625 1.5H9a3.75 3.75 0 013.75 3.75v1.875c0 1.036.84 1.875 1.875 1.875H16.5a3.75 3.75 0 013.75 3.75v7.875c0 1.035-.84 1.875-1.875 1.875H5.625a1.875 1.875 0 01-1.875-1.875V3.375c0-1.036.84-1.875 1.875-1.875zM12.75 12a.75.75 0 00-1.5 0v2.25H9a.75.75 0 000 1.5h2.25V18a.75.75 0 001.5 0v-2.25H15a.75.75 0 000-1.5h-2.25V12z" clip-rule="evenodd" />
							<path d="M14.25 5.25a5.23 5.23 0 00-1.279-3.434 9.768 9.768 0 016.963 6.963A5.23 5.23 0 0016.5 7.5h-1.875a.375.375 0 01-.375-.375V5.25z" />
						</svg>
				</div>
			</button>
				
			{/if}
		</div>
	</div>  


			<div class="relative mt-8 ml-8  flex flex-wrap justify-center">
				<!-- lookup prompt -->                                
				<form class="mx-auto w-10/12 sm:w-[600px] md:w-[650px] lg:w-[800px] -mb-12">
					<label class="w-full flex flex-wrap items-stretch relative">
						
						<div class="w-4/5 mx-auto flex-row flex" >
							<Hint placement="bottom" text="Show All Decks">
								<div style="text-align-last:center;" class="border-y border-r bg-white justify-center w-12 h-8 border-l rounded-l-lg relative z-30 items-center outline-columbia focus:outline-none outline-4 pl-1 font-bold text-gray-500 transition-all border-gray-200 cursor-pointer group ease border-columbia overflow-hidden duration-75">
										<div class="focus:ring cursor-pointer ring-columbia">
											<input
											class="cursor-pointer ml-2 mr-4 ring-columbia accent-columbia focus:ring mt-2 h-4 w-4  {panel.display_all ? "" : ""}"
											type=checkbox bind:checked={panel.display_all}>

										</div>
								</div>
							</Hint>
							<input type="text"
								bind:value={panel.prompt}
								on:change={filterCards}
								class="pl-2 h-8 placeholder:font-italic border-y border-r rounded-r-lg w-full border-columbia py-2 pr-10 focus:outline-none"
								placeholder=""/>

						</div>
					</label>
				</form>
				
			</div>
		<div>

	</div>
		

</div>
	<!-- gallery of cards -->
	<div class=" bg-offwhite dark:bg-offblack h-full">
		<div class="h-16"></div>
		
		<div class="flex flex-col gap-3">   
			<!-- Note: the keyed index must be (card) for the animation to work -->
			{#each card_gallery as card, index (card)}

				<div
					animate:flip={{ duration: dragDuration }}
					style="display: {card.is_visible ? '' : 'none'};"
					class="rounded-lg text-offblack dark:text-offwhite mx-auto w-10/12 sm:w-[600px] md:w-[650px] lg:w-[800px] dark:bg-opacity-70 border-columbia bg-slate-700 bg-opacity-5 "
					draggable="true"
					on:dragstart={() => draggingCard = card}
					on:dragend={() => draggingCard = undefined}
					on:dragenter={() => swapWith(card)}
					on:dragover|preventDefault
				>
					<!-- card fields -->
				<div class="flex flex-row">
					<div class="flex flex-col gap-0 py-3 pr-2 pl-2 w-full h-full">
						<div class="flex flex-row space-x-0 w-full h-full ">
							<div class="w-1/2 flex-1 border-x border-t { !card.show_expl ?  "border-b" : "rounded-b-none" } rounded-lg border-dotted border-columbia">
								<div on:focusout={() => updateCard(card.card)} class="w-full px-1 py-2 rounded-lg">
									<TextfieldEditor bind:content={card.card.front} is_gallery={true}/>
								</div>
							</div>						
							

							<div class="w-1/2 flex-1 border-r border-t { !card.show_expl ?  "border-b" : "rounded-b-none" } rounded-lg border-dotted border-columbia">
								<div on:focusout={() => updateCard(card.card)} class="w-full  px-2 py-2 rounded-lg">
									<TextfieldEditor bind:content={card.card.back} is_gallery={true}/>
								</div>	
							</div>
						</div>
						

						{#if card.show_expl}
							<div class="">
								<div on:focusout={() => updateCard(card.card)} class="w-full opacity-90 border-x border-y rounded-b-lg rounded-tr-sm border-dotted border-columbia border-spacing-4 px-2 py-2">
									<TextfieldEditor bind:content={panel.expl} is_expl={true}/>
								</div>					
							</div>
						{/if}
						

					</div>
					<div class="flex flex-col justify-between -mr1">
						<span class="cursor-pointer hover:opacity-100 pt-3 right-1 opacity-50 text-md relative text-columbia" on:click={() => deleteCard(card)} on:keydown={() => deleteCard(card)}> 
							<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
								<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
							  </svg>
							  
						</span>

						<span class="cursor-pointer hover:opacity-100 pb-1 right-2 pl-1 bottom-2 opacity-50 relative text-md text-columbia" on:click={() => toggleShowExplanation(card)} on:keydown={() => toggleShowExplanation(card)}> 
							{#if !card.show_expl}
								<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
									<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 5.25l-7.5 7.5-7.5-7.5m15 6l-7.5 7.5-7.5-7.5" />
								</svg>
							{:else}
								<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
									<path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l7.5-7.5 7.5 7.5m-15 6l7.5-7.5 7.5 7.5" />
								</svg>
							
							{/if}
							
						</span>

					 </div>

				</div>
					

				</div>

			{/each}
		</div>
		<!-- <div class="flex flex-row absolute bottom-0 left-4"	></div> -->
	</div>
	</div>

	<!-- <div class="h-32"></div> -->

</div>


</div>

<style >
	.card {                                                                     
		box-shadow: 0 10px 20px -8px rgba(197, 214, 214);                       
        transition: all 0.3s cubic-bezier(0, 0, 0.5, 1);                                                                    
    }                                                                           


</style> 
