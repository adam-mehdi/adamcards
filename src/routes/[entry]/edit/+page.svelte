
<head>
	<link
	rel="stylesheet"
	href="https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/katex.min.css"
	integrity="sha384-vKruj+a13U8yHIkAyGgK1J3ArTLzrFGBbBc0tDp4ad/EyewESeXE/Iv67Aj8gKZ0"
	crossorigin="anonymous"
/>
</head>
<script lang="ts">
	import { flip } from 'svelte/animate'
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onDestroy } from 'svelte';
	import {clickOutside} from '$lib/actions/click_outside.js';
	import configStore from '$lib/stores/configStore'
	import Editor from '@tinymce/tinymce-svelte';
	import Hint from 'svelte-hint';
	import { preprocess, text_patterns, apiKey } from '$lib/editor';
	const scriptSrc = "/src/lib/tinymce/js/tinymce/tinymce.min.js";


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

		// entryChildren.deck_names = entryChildren.deck_names.map(s => s.replace("~~", "/"))
		deck_names = entryChildren.deck_names;
		deck_fnames = deck_names.map((x: string) => path2name(x));
		panel.selected_deck = deck_fnames[0];
		
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
		const front = panel.front.replaceAll("\n", "");
		const back = panel.back.replaceAll("\n", "");;

		let deckName = deck_names.filter((x: string) => x.endsWith(panel.selected_deck))[0];
		const id: number = await invoke(
			"calculate_hash", 
			{"deckName": deckName, "front": front, "back": back }
			);
		const new_card: Card = {
			"fcard": {
				"id": id, 
				"front": front, 
				"back": back, 
				"deck_name": deckName,
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
			if (cs.fcards.length < 16 || !panel.display_multi) {
				cs.fcards.splice(0, 0, new_card.fcard);
				// cs.fcards.push(new_card.fcard);
			}
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
		let pairs = parse_textfield();
		
		panel.making_multi = false;
		let front_temp = panel.front;
		let back_temp = panel.back;

		for (const pair of pairs) {
			panel.front = pair.front; 
			panel.back = pair.back; 
			await createCard() 
		}
		
		panel.front = front_temp;
		panel.back = back_temp;
		panel.making_multi = true;
		panel.textfield = '';

		// save all cards created by textfield
		await saveDecks()
		return;

	}

	function parse_textfield(): FieldPair[] {
		if (panel.textfield == "")
			return [];

		let txt = panel.textfield;
		txt = txt.replaceAll("&bull; ", "");
		let pairs: FieldPair[] = [];

		let results = txt.match(/<.*?>*?(&rarr;)*?<\/.*>/g);

		if (results) {
			for (let result of results) {
				// remove div; make the latter part dynamic
				result = result.substring(result.indexOf(">")+1, result.lastIndexOf("<"));
				let style_match = result.match(/(style=").*?"/g);
				if (style_match != null)
					result = result.replaceAll(style_match[0], "");
				
				
				for (let line of result.split("<br>")) {
					let card = line.replaceAll("&rarr;", "⟶").split("⟶");
					if (card && card.length == 2) {
						let front = `<div>${card[0].trim()}</div>`;
						let back = `<div>${card[1].trim()}</div>`;
						pairs.push({ front, back });
					}
				}
			}
		}

		return pairs;
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
	
	function filterCards() {	
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

	const dragDuration = 150
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
		panel.selectDeckCreate = false;
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



	let inline_conf = {
		skin: isDarkMode ? "oxide-dark": "oxide",
		menubar: false,
		toolbar: false,
		content_style: 'img {object-fit: cover; width: 100%; border-radius: 5%; display: block; margin-left: auto; margin-right: auto;}',
		plugins: 'lists',
		branding: false,
		text_patterns: text_patterns,
		paste_preprocess: preprocess
	}

	let conf = {
		menubar: false,
		min_height: 75,
		height: 150,
		max_height: 200,
		resize: true,
		plugins: 'lists',
		toolbar: false,
		toolbar_location: "bottom",
		fullscreen_native: true,
		branding: false,
		elementpath: false,
		skin: isDarkMode ? 'oxide-dark' : "oxide",
		content_css: isDarkMode ? "dark" : "",
		content_style: 'img {object-fit: cover; max-width: 60%; border-radius: 5%; display: block; margin-left: auto; margin-right: auto;}',
		text_patterns: text_patterns,
		paste_preprocess: preprocess
	} 

	let textfield_conf = {
		menubar: false,
		min_height: 75,
		height: 300,
		max_height: 600,
		resize: true,
		plugins: 'fullscreen image lists',
		// plugins: 'fullscreen image editimage lists',
		toolbar: 'styles fontsize forecolor indent outdent bullist numlist paste fullscreen ',
		font_size_formats: '8pt 10pt 12pt 14pt 16pt 24pt',
		toolbar_location: "bottom",
		fullscreen_native: true,
		branding: false,
		elementpath: false,
		skin: isDarkMode ? 'oxide-dark' : "oxide",
		content_css: isDarkMode ? "dark" : "",
		content_style: 'img {object-fit: cover; max-width: 60%; border-radius: 5%; display: block; margin-left: auto; margin-right: auto;}',
		plugin: 'lists',
		text_patterns: text_patterns,
 		indent_use_margin: true,
		forced_root_block : 'div',

		// setup command to process math
		// setup: (ed: any) => {
		// 	// replace text inside of $$ with inline katex math
		// 	ed.addCommand('math', (ui: any, v: any) => {
		// 		ed.windowManager.alert('Hello world!! Selection: ' + ed.selection.getContent({ format: 'text' }));
		// 	});
		// },

		paste_preprocess: preprocess
	}

	$: panel.selected_deck, filterCards()

	

</script>

	
<div class="{isDarkMode ? "dark" : ""}">
<div class="min-h-screen h-full bg-offwhite dark:bg-offblack">
		
		<!-- home button -->
		<div class="flex justify-start space-x-5 mx-10">
			
			<!-- home button -->
			<a href="/" class="outline-columbia  ring-columbia mt-1 focus:outline-none focus:ring duration-75 rounded-md">
				<div on:click={saveDecks} on:keypress={saveDecks} class="fled justify-evenly ring-columbia  focus:outline-none focus:ring duration-75 rounded-md">
					<svg 
						fill="highlight" class="fill-columbia float-left h-7 w-7 cursor-pointer outline-columbia focus:outline outline-4 outline-offset-2 " 
						viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path clip-rule="evenodd" fill-rule="evenodd" d="M9.293 2.293a1 1 0 011.414 0l7 7A1 1 0 0117 11h-1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-3a1 1 0 00-1-1H9a1 1 0 00-1 1v3a1 1 0 01-1 1H5a1 1 0 01-1-1v-6H3a1 1 0 01-.707-1.707l7-7z"></path>
					</svg>
				</div>
			</a>

			<div class="bg-offwhite z-50 dark:bg-offblack">
				{#if !panel.display_multi}
					<!-- front-back -->
					<button 
						class="h-9 w-9 ring-columbia mt-1  focus:outline-none focus:ring duration-75 rounded-md"
						on:click={toggleMulti}>
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
				<button on:click={toggleMulti} class="h-9 w-9 z-50 dark:bg-offblack ring-columbia mt-1 focus:outline-none focus:ring duration-75 rounded-md">
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

		</div>

	 <!-- central panel -->
	<div class="mt-4 h-full flex flex-col justify-center items-center">
		<!-- card fields -->
		<div class="card flex flex-col h-full rounded-lg text-blacktext w-10/12 sm:w-[600px] md:w-[650px] lg:w-[800px] dark:text-whitetext">

			{#if !panel.display_multi}
				<!-- front field -->
				<div class="h-full max-h-80 mx-2 mb-1 border-l rounded-md border-columbia" >         
				  <!-- md:w-[700px] lg:w-[800px] -->
					<div class="h-full p-1 rounded-lg ">
						<Editor {conf} inline={false} bind:value={panel.front} {scriptSrc}/>

					</div>
				</div>          

				<!-- back field -->
				<div class="h-full max-h-80 mx-2 border-l rounded-md mb-1 border-columbia" >         
					<div class="h-full p-1 rounded-lg ">
						<Editor {conf} inline={false} bind:value={panel.back} {scriptSrc}/>
					</div>
				</div>          
					

			{:else}
				<div class="h-full m-3 mb-1 text-inherit" >         
					<div class="h-full p-1 rounded-lg ">
						<Editor conf={textfield_conf} inline={false} bind:value={panel.textfield} {scriptSrc}/>
					</div>
				</div>
			{/if}
			
			<!-- make this one bar -->
			<div class="flex items-center justify-center top-[450px]">     

				<button 
					on:click={createCard} 
					class="h-8 w-1/4 relative z-50 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all border-y border-l border-gray-200 rounded-bl-lg cursor-pointer group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75">
					<!-- <svg class="fill-columbia flex-none h-5 w-6 outline-columbia focus:outline outline-4 outline-offset-2 "
						 viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path d="M5.127 3.502L5.25 3.5h9.5c.041 0 .082 0 .123.002A2.251 2.251 0 0012.75 2h-5.5a2.25 2.25 0 00-2.123 1.502zM1 10.25A2.25 2.25 0 013.25 8h13.5A2.25 2.25 0 0119 10.25v5.5A2.25 2.25 0 0116.75 18H3.25A2.25 2.25 0 011 15.75v-5.5zM3.25 6.5c-.04 0-.082 0-.123.002A2.25 2.25 0 015.25 5h9.5c.98 0 1.814.627 2.123 1.502a3.819 3.819 0 00-.123-.002H3.25z"></path>
					</svg> -->
					<svg class="fill-columbia stroke-columbia flex-none h-6 w-6 outline-columbia focus:outline outline-4 outline-offset-2 "
						stroke="currentColor" stroke-width="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15"></path>
					</svg>
					<!-- Create -->
				</button>

				<!-- select deck -->
				<div style="text-align-last:center;" class="border-y px-3 py-1 w-1/2 h-8 relative z-30 outline-columbia focus:outline outline-4 outline-offset-2 overflow-hidden font-bold text-gray-500 transition-all duration-200 border-gray-200 cursor-pointer group ease border-columbia  bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white">
					<select bind:value={panel.selected_deck}
						class="text-columbia relative appearance-none w-full h-full outline-columbia focus:outline outline-4 outline-offset-2 justify-center z-30 inline-flex overflow-hidden font-bold text-gray-500 border-gray-200 cursor-pointer group ease border-columbia bg-gradient-to-b from-offwhite dark:from-offblack dark:to-offblack dark:border-x dark:rounded-none to-gray-50 hover:from-gray-50 hover:to-white active:to-white transition-all duration-100"
					>

						{#each deck_fnames as deck_fname}
							<option value={deck_fname}>
								{deck_fname}
							</option>
						{/each}	
					</select>
				</div>



				<!-- Undo button -->
				{#if cs.rm_stack.length > 0}
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
					<!-- Undo -->
						<svg class="fill-columbia flex-none h-8 w-8 outline-columbia focus:outline outline-4 outline-offset-2"
							viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
							<path clip-rule="evenodd" fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"></path>
						</svg>
					</button>
				{/if}
			</div>
				
  

		</div>  


		

			<div class="relative mt-8 ml-8  flex flex-wrap justify-center">

				<!-- lookup prompt -->                                
				<form class="mx-auto w-10/12 sm:w-[600px] md:w-[650px] lg:w-[800px] -mb-12">
					<label class="w-full flex flex-wrap items-stretch relative">
						
						<div class="w-4/5 mx-auto flex-row flex" >
							<div style="text-align-last:center;" class="border-y border-r bg-white justify-center w-1/4 h-8 border-l rounded-l-lg relative z-30 items-center outline-columbia focus:outline-none outline-4 pr-2 pl-1 py-1 font-bold text-gray-500 transition-all border-gray-200 cursor-pointer group ease border-columbia overflow-hidden ring-columbia focus:ring duration-75">
								<select bind:value={panel.lookup_deck}
									class="text-columbia relative appearance-none w-full h-full z-30 inline-flex overflow-hidden font-bold text-gray-500 border-gray-200 cursor-pointer group ease ring-columbia focus:outline-none ring-offset-2 rounded-none rounded-l focus:ring duration-75"
								>
									<option value={""}>
										All Decks
									</option>
			
									{#each deck_fnames as deck_fname}
										<option value={deck_fname}>
											{deck_fname}
										</option>
									{/each}	
								</select>
							</div>



							<input type="text"
							bind:value={panel.prompt}
							class="pl-2  h-8 placeholder:font-italic border-y border-r rounded-r-lg w-full border-columbia py-2 pr-10 focus:outline-none"
							placeholder=""/>
							
							<button on:click={filterCards} on:keydown={filterCards} class="absolute right-2 sm:right-8 lg:right-14 mr-10 top-1 outline-columbia focus:outline outline-4 outline-offset-2 transition-all active:bg-columbia ring-columbia focus:outline-none ring-offset-2 rounded-r focus:ring duration-75">
								<svg class="h-6 w-6 fill-none stroke-columbia"
									stroke="currentColor" stroke-width="1.75" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
									<path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607zM10.5 7.5v6m3-3h-6"></path>
								</svg>
							</button>

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
			{#each cs.fcards as card, index (card)}
				<div
					animate:flip={{ duration: dragDuration }}
					class="flex flex-row rounded-lg text-offblack dark:text-offwhite mx-auto w-10/12 sm:w-[600px] md:w-[650px] lg:w-[800px]  dark:bg-opacity-70 border-columbia bg-slate-700 bg-opacity-5 "
					draggable="true"
					on:dragstart={() => draggingCard = card}
					on:dragend={() => draggingCard = undefined}
					on:dragenter={() => swapWith(card)}
					on:dragover|preventDefault
				>

					<!-- card fields -->
						
						<div class="ml-3 mr-0 w-1/2 m-3 rounded-lg border-l border-columbia border-spacing-4 px-4 py-2">
							<Editor bind:value={card.front} inline={true} conf={inline_conf} {scriptSrc}/>
						</div>

						<div class="border-r-2 opacity-30 border-columbia"></div>

						<!-- <div class="card-hr mt-5" /> -->
						<div class="ml-0 mr-4 w-1/2 m-3 border-r rounded-lg border-columbia border-spacing-4 px-4 py-2">
							<Editor bind:value={card.back} inline={true} conf={inline_conf} {scriptSrc}/>
						</div>

						<span 
							class="cursor-pointer float-right right-2 relative text-columbia" 
							on:click={() => deleteCard(card)} 
							on:keydown={() => deleteCard(card)}
							> 
							✕
						</span>
<!-- 					
					<div class="text-sm h-16 m-3">
						<textarea 
							class="w-full h-16 rounded-lg resize-none outline-offset-2 bg-offwhite dark:bg-offblack text-inherit"
							bind:value={card.back} />
					</div> -->
					
				</div>
			{/each}
		</div>
		<div class="flex flex-row absolute bottom-0 left-4"	>
				
			</div>
		<div class="dark:text-offwhite">
			{panel.textfield}
		</div>
	</div>

</div>


	<!-- <div class="h-96" /> -->
</div>

<style >
	.card {                                                                     
		box-shadow: 0 10px 20px -8px rgba(197, 214, 214);                       
        transition: all 0.3s cubic-bezier(0, 0, 0.5, 1);                                                                    
    }                                                                           



</style>