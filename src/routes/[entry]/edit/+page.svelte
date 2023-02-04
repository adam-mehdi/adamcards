<script lang="ts">
	import { flip } from 'svelte/animate'
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onDestroy } from 'svelte';
	import {clickOutside} from '$lib/actions/click_outside.js';
	import configStore from '$lib/stores/configStore'

	// contains state for the central panel on which cards are created
	interface CenterPanel {
		front: string,
		back: string,
		prompt: string,
		selected_deck: string,
		lookup_deck: string,
		display_multi: boolean,
		making_multi: boolean,
		textfield: string,
		selectDeckLookup: boolean,
		selectDeckCreate: boolean
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
	let isDarkMode: boolean = $configStore.is_dark_mode;

	// init panel
	let panel: CenterPanel = {
		"front": '',
		"back": '',
		"prompt": '',
		"display_multi": $configStore.is_textfield,
		"making_multi": $configStore.is_textfield,
		"textfield": '',
		"selectDeckLookup": false,
		"lookup_deck": '',
		"selectDeckCreate": false,
		"selected_deck": '',
	};

	let cs: CardState = {
		"card_map": new Map<number, Card>(),
		"fcards": [],
		"rm_stack": []
	}

	// all deck children of provided entry 
	let deck_names: string[] = [];
	let deck_fnames: string[] = [];
	let no_decks: boolean = false;

	async function getDecks() {
		let entryChildren: EntryChildren = await invoke(
			'read_decks', 
			{ "entry": $page.params.entry }
			);

		if (entryChildren.deck_names.length == 0) {
			no_decks = true;
			return;
		}

		
		// extract deck names that are children of file system entry

		entryChildren.deck_names = entryChildren.deck_names.map(s => s.replace("~~", "/"))
		deck_names = entryChildren.deck_names;
		deck_fnames = deck_names.map((x: string) => path2name(x));
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
			await createCardTextfield()
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
			if (cs.fcards.length < 25)
				cs.fcards.splice(0, 0, new_card.fcard);
			else // push for efficiency's sake
				cs.fcards.push(new_card.fcard);

			cs.fcards = cs.fcards;
		}

		// cleanup panel fields
		panel.front = '';
		panel.back = '';

		// save all cards every card created if front/back
		if (!panel.display_multi) 
			await saveDecks()
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
			await createCard() 
		}
		
		panel.front = front_temp;
		panel.back = back_temp;
		panel.making_multi = true;
		panel.textfield = '';

		// save all cards created by textfield
		console.log("saving decks");
		await saveDecks()
		return;

	}


	function toggleMulti() {
		panel.display_multi = !panel.display_multi;
		panel.making_multi = !panel.making_multi;
	}

	$: configStore.update((config) => {
		config.is_textfield = panel.display_multi;
		return config;
	})

	onDestroy(async () => {
		await invoke('write_global_config', { "config": $configStore });
	});

	function get_is_displayed(fcard: FrontendCard): boolean {
		let prompt = panel.prompt.toLowerCase();
		if (fcard.front.toLowerCase().includes(prompt) ||
			fcard.back.toLowerCase().includes(prompt)) {
				let deck_fname = path2name(fcard.deck_name);
				if (panel.lookup_deck == '' || deck_fname == panel.lookup_deck)
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
			cs.fcards.push(new_card.fcard);
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

	function handleClickOutside() {
		panel.selectDeckLookup = false;
		panel.selectDeckCreate = false;
	}


	function handleSelectedDeckLookup(fname: string) {
		panel.selectDeckLookup = false; 
		panel.lookup_deck = fname;
		filterCards();
	}

	function getDuplicateDeckNamesExist(): boolean {
		let ancestors_set = deck_names.map((x: string) => x.split("~~"));
		let names = ancestors_set.map((x: string[]) => x[x.length - 1]);
		let hasDuplicates = (new Set(names)).size !== names.length;
		return hasDuplicates;
	}
	let duplicateDeckNamesExist = getDuplicateDeckNamesExist()

	function path2name(deck_path: string): string {
		let ancestors = deck_path.split("~~");
		if (duplicateDeckNamesExist)
			return ancestors[ancestors.length - 2] + "/" + ancestors[ancestors.length - 1];
		else
			return ancestors[ancestors.length - 1];
	}

	function handleSelectedDeckCreate(fname: string) {
		panel.selected_deck = deck_names.filter((x: string) => x.endsWith(fname))[0];
	}

	
</script>


	<!-- choose deck name; `selected_deck_name` by default -->
	
<div class="{isDarkMode ? "dark" : ""}">
<div class="min-h-screen h-full bg-offwhite dark:bg-offblack">
		<!-- home button -->
		<div class="flex justify-between mb-5 mx-10">
			<!-- home button -->
			<a href="/" class="outline-columbia focus:outline outline-4 outline-offset-2 ">
				<div on:click={saveDecks} on:keypress={saveDecks} class="fled justify-evenly mt-2 ">
					<svg 
						fill="highlight" class="flex-none h-7 w-7 cursor-pointer outline-columbia focus:outline outline-4 outline-offset-2 " 
						viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path clip-rule="evenodd" fill-rule="evenodd" d="M9.293 2.293a1 1 0 011.414 0l7 7A1 1 0 0117 11h-1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-3a1 1 0 00-1-1H9a1 1 0 00-1 1v3a1 1 0 01-1 1H5a1 1 0 01-1-1v-6H3a1 1 0 01-.707-1.707l7-7z"></path>
					</svg>
				</div>
			</a>
		</div>


	 <!-- central panel -->
	<div class="mt-20 h-80 flex flex-col justify-center items-center ">
		{#if no_decks}
			<h2 class="font-mono">No decks found. Must create deck under 
				<span class="font-bold">{$page.params.entry}</span>.</h2>
		{/if}

		

		<!-- card fields -->
		<div class="card flex flex-col h-full rounded-lg text-blacktext dark:text-whitetext">
			<div class="flex items-center justify-start h-4 top-[450px] mb-2">
			
				<!-- Dropdown menu -->
					{#each deck_fnames as deck_fname}
						<button	
							on:click={() => handleSelectedDeckCreate(deck_fname)} 
							class="h-6 w-full first:rounded-tl-lg last:rounded-tr-lg relative z-30 text-sm overflow-hidden text-blacktext transition-all duration-500 group ease bg-gradient-to-b from-offwhite to-platinum hover:from-platinum hover:to-offwhite  border-t border-platinum {path2name(panel.selected_deck) == deck_fname ? "cursor-default from-columbia to-columbia opacity-50 font-extrabold" : "cursor-pointer font-semibold from-platinum to-offwhite dark:to-platinum" } outline-columbia focus:outline outline-4 outline-offset-2"
							>
								<span class="w-full h-0.5 absolute bottom-0 group-active:bg-transparent left-0 bg-gray-100"></span>
								<span class="h-full w-0.5 absolute bottom-0 group-active:bg-transparent right-0 bg-gray-100"></span>
								{deck_fname}
						</button>    
							
					{/each}
			</div>

			{#if !panel.display_multi}
				<!-- front field -->
				<div class="h-24 w-[520px] lg:w-[700px] m-2 text-inherit" >         
				  <!-- md:w-[700px] lg:w-[800px] -->
					<textarea          
						class="h-24 w-[520px] lg:w-[700px] rounded-lg resize-none bg-offwhite  dark:bg-offblack text-inherit outline-columbia focus:outline outline-4 outline-offset-2 "                                                    
						bind:value={panel.front}                                    
					/>                                                              
				</div>          

				<!-- rule separating front and back fields -->
				<div class="border-t border-1 border-platinum" />   

				<!-- back field -->
				<div class="h-24 w-[500px] lg:w-[700px] m-2 text-inherit" >         
					<!-- md:w-[700px] lg:w-[800px] -->
					  <textarea          
						  class="h-24 w-[520px] lg:w-[700px] rounded-lg resize-none bg-offwhite  dark:bg-offblack text-inherit outline-columbia focus:outline outline-4 outline-offset-2"                                                    
						  bind:value={panel.back}                                    
					  />                                                              
				</div>          
					

			{:else}
				<div class="h-50 w-[520px] lg:w-[700px] m-3 text-inherit" >         
					<!-- md:w-[700px] lg:w-[800px] -->
					  <textarea          
						  class="h-48 w-[520px] rounded-lg lg:w-[700px] resize-none bg-offwhite  dark:bg-offblack text-inherit outline-columbia focus:outline outline-4 outline-offset-2"                                                    
						  bind:value={panel.textfield}                                   
					  />                                                              
				</div>          

			{/if}
			
			<!-- make this one bar -->
			<div class="flex items-center justify-center top-[450px]">     

				<button 
					on:click={createCard} 
					class="h-5 w-1/3 relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all duration-500 border-y border-l border-gray-200 rounded-bl-lg cursor-pointer group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white">
					<span class="w-full h-0.5 absolute bottom-0 group-active:bg-transparent left-0 bg-gray-100"></span>
					<span class="h-full w-0.5 absolute bottom-0 group-active:bg-transparent right-0 bg-gray-100"></span>
					Create
				</button>


				<button
					on:click={toggleMulti}
					class="h-4 w-1/3 relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all duration-500 border border-gray-200 cursor-pointer group ease bg-gradient-to-b from-white border-columbia to-gray-50 hover:from-gray-50 dark:from-offblack hover:to-white active:to-white outline-columbia focus:outline outline-4 outline-offset-2">
					<span class="w-full h-0.5 absolute bottom-0 group-active:bg-transparent left-0 bg-gray-100"></span>
					<span class="h-full w-0.5 absolute bottom-0 group-active:bg-transparent right-0 bg-gray-100"></span>
					{!panel.display_multi ? `Textfield` : `Front/Back`}
				</button>      

				<button	
					on:click={undoDelete} 
					class="h-4 w-1/3 relative z-30 inline-flex rounded-br-lg items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 {cs.rm_stack.length == 0 ? "opacity-30 cursor-default" : "cursor-pointer" } transition-all duration-500 border-columbia border dark:from-offblack group ease bg-gradient-to-b from-white to-gray-50 hover:from-gray-50 hover:to-white active:to-white outline-columbia focus:outline outline-4 outline-offset-2">
					<span class="w-full h-0.5 absolute bottom-0 group-active:bg-transparent left-0 bg-gray-100"></span>
					<span class="h-full w-0.5 absolute bottom-0 group-active:bg-transparent right-0 bg-gray-100"></span>
					Undo
				</button>    

			</div>     

		</div>  


		

			<div class="relative mt-8 flex flex-wrap justify-center w-full">

				<!-- lookup prompt -->                                
				<form class="w-full">
					<label class="w-full flex flex-wrap items-stretch relative mb-3">
						<span class="absolute left-1/4 inset-y-0 flex items-center pl-3">
							<svg class="h-5 w-5 fill-black" xmlns="http://www.w3.org/2000/svg" x="0px" y="0px" width="30"
								height="30" viewBox="0 0 30 30">
								<path
									d="M 13 3 C 7.4889971 3 3 7.4889971 3 13 C 3 18.511003 7.4889971 23 13 23 C 15.396508 23 17.597385 22.148986 19.322266 20.736328 L 25.292969 26.707031 A 1.0001 1.0001 0 1 0 26.707031 25.292969 L 20.736328 19.322266 C 22.148986 17.597385 23 15.396508 23 13 C 23 7.4889971 18.511003 3 13 3 z M 13 5 C 17.430123 5 21 8.5698774 21 13 C 21 17.430123 17.430123 21 13 21 C 8.5698774 21 5 17.430123 5 13 C 5 8.5698774 8.5698774 5 13 5 z">
								</path>
							</svg>
						</span>
						<div class = "w-1/2 mx-auto">
							<input
							bind:value={panel.prompt} 
							on:input={filterCards} 
							class=" rounded-lg h-8 placeholder:font-italic border w-full border-columbia py-2 pl-10 pr-4 focus:outline-none"
							placeholder="Filter Cards" type="text" />

						</div>
					</label>
				</form>
				
				
				<!-- select deck to filter by -->
				{#if deck_names.length > 1}
				<div class="z-50 h-8 rounded-r-lg leading-snug font-normal text-center text-slate-300 border-columbia border-r border-y absolute justify-center right-1/4 outline-columbia focus:outline outline-4 outline-offset-2"
					use:clickOutside 
					on:click_outside={handleClickOutside}
				>
					<button 
						on:click={() => {panel.selectDeckLookup = !panel.selectDeckLookup;}}
						class="flex items-center h-8 pl-3 pr-2 outline-columbia focus:outline outline-4 outline-offset-2">
						{#if panel.lookup_deck == ''}
							<span class="text-sm leading-none">All Decks</span>
						{:else}
							<span class="text-sm leading-none">{path2name(panel.lookup_deck)}</span>
						{/if}
						<svg class="w-4 h-4 mt-px ml-2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
							<path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
						</svg>
					</button>
					{#if panel.selectDeckLookup}
					<div class="absolute flex flex-col w-28 shadow-lg leading-snug">
						{#if panel.lookup_deck != ''}
						<button 
							class="flex items-center h-8 px-3 w-full text-sm hover:bg-gray-200 outline-columbia focus:outline outline-4 outline-offset-2" 
							on:click={() => handleSelectedDeckLookup('')}
							>All Decks</button>
						{/if}
						{#each deck_fnames as deck_fname}
							{#if path2name(panel.lookup_deck) != deck_fname}
							<button 
								class="flex last:rounded-b-lg first:rounded-t-lg items-center h-8 px-3 text-sm hover:bg-gray-200 scroll-smooth bg-offwhite dark:bg-offblack z-50 hover:bg-platinum outline-columbia focus:outline outline-4 outline-offset-2"
								on:click={() => handleSelectedDeckLookup(deck_fname)}>
								{deck_fname}
							</button>
							{/if}
						{/each}
					</div>
					{/if}
				</div>
				{/if}
				<!-- deck dropdown end -->
			</div>
		<div>

	</div>
		

</div>
	<!-- gallery of cards -->
	<div class=" bg-offwhite dark:bg-offblack h-full">
		<div class="h-16"></div>
		<div class="card-container ">   
			<!-- Note: the keyed index must be (card) for the animation to work -->
			{#each cs.fcards as card (card)}
				<div
					animate:flip={{ duration: dragDuration }}
					class="card rounded-lg bg-offwhite dark:bg-offblack text-offblack dark:text-offwhite "
					draggable="true"
					on:dragstart={() => draggingCard = card}
					on:dragend={() => draggingCard = undefined}
					on:dragenter={() => swapWith(card)}
					on:dragover|preventDefault
				>
					<span 
						class="cursor-pointer float-right mr-2 bottom-1 relative text-platinum" 
						on:click={() => deleteCard(card)} 
						on:keydown={() => deleteCard(card)}
						> 
						✕
					</span>

					<!-- card fields -->
					<div class="mx-4 text-sm h-16 m-3">
						<textarea 
							class="w-full h-16 rounded-lg resize-none outline-offset-1 bg-offwhite dark:bg-offblack text-inherit"
							bind:value={card.front} />
					</div>
					<div class="card-hr mt-5" />
					
					<div class="text-sm h-16 m-3">
						<textarea 
							class="w-full h-16 rounded-lg resize-none outline-offset-2 bg-offwhite dark:bg-offblack text-inherit"
							bind:value={card.back} />
					</div>
					
				</div>
			{/each}
		</div>
	</div>

</div>

	<!-- <div class="h-96" /> -->
</div>

<style >
                                                                                

    .card-container {                                                           
                                                                                 
        display: grid;                                                          
        grid-template-columns: repeat(auto-fit, minmax(300px, max-content));    
        grid-gap: 32px;                                                         
        justify-content: center;                                                
        padding: initial;                                                       
    }                       

	.card {                                                                     
		box-shadow: 0 10px 20px -8px rgba(197, 214, 214);                       
        transition: all 0.3s cubic-bezier(0, 0, 0.5, 1);                                                                    
    }                                                                           
                                                                                
    .card-hr {                                                                  
        border-top: 1px solid #e1dfdd;                                          
    }                                                                           
                                                                                
                                                          
	


</style>