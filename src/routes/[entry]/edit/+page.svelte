
<script lang="ts">
	import { flip } from 'svelte/animate'
	import { page, updated } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';
	import configStore from '$lib/stores/configStore'
	import Hint from 'svelte-hint';
	import {clickOutside} from '$lib/actions/click_outside.js';

	import { onMount, onDestroy } from 'svelte'
	import TextfieldEditor from '$lib/TextfieldEditor.svelte'
  	import type { ChatCompletionRequestMessage } from 'openai';
	// import { processChatRequest } from '$lib/chatProcessor';

	


	// contains state for the central panel on which cards are created
	interface CenterPanel {
		front: string,
		back: string,
		explanation: string,
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
		back: string,
		explanation: string
	}

	interface CardDisplay {
		is_visible: boolean,
		deck_name: string,
		show_explanation: boolean,
		loading: boolean,
		is_suggested: boolean,
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
		"explanation": '',
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

	function scrollToTop() {
		setTimeout(function () {
			scrollTo(0, 0)
		}, 100)
		setTimeout(function () {
			scrollTo(0, 0)
		}, 200)
		setTimeout(function () {
			scrollTo(0, 0)
		}, 300)
		setTimeout(function () {
			scrollTo(0, 0)
		}, 500)
	}


	async function getDeadlineContents() {

		deadlineContents = await invoke('read_deadline_contents', { deadlineId });
		if (deadlineContents.length == 0) return
		panel.selected_deck = deadlineContents[deadlineContents.length - 1].deck_name.toString();

		for (let deckContents of deadlineContents) {
			for (let card of deckContents.cards) {
				let card_display: CardDisplay = { 
					"is_visible": false, 
					"deck_name": deckContents.deck_name,
					"show_explanation": false,
					"loading": false,
					"is_suggested": false,
					card 
				};
				card_gallery.splice(0,0,card_display);
			}
		}
		filterCards()


		if (deadlineContents.length == 0) 
			createDeckTrayOpen = true;
		scrollToTop()
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
				"back": new_card.back,
				"explanation": ""
			}
			deckContents.cards.push(card);

			let cardDisplay = { 
				"is_visible": false, 
				"is_suggested": false,
				"deck_name": deckContents.deck_name,
				"card": card,
				"show_explanation": true,
				"loading": false
			}
			cardDisplay.is_visible = get_is_displayed(cardDisplay)
			card_gallery.splice(0, 0, cardDisplay)
			getExplanation(cardDisplay)
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
		let questions = txt.split("</div><div>");
		let newCards: NewCard[] = [];
		for (let question of questions) {
			let [front, back] = stripHtml(question).split("»");
			if (front == '' || !back) continue
			newCards.push({front, back})
		}

		// if (!txt.startsWith("<div>"))
		// 	txt = "<div>" + txt
		// if (!txt.endsWith("</div>"))
		// 	txt = txt + "</div>"
		// txt = txt.replaceAll("•", "");

		// let results = txt.match(/<(div|ul|li)>.*?».*?<\/(div|ul|li)>/g);
		// // let results = txt.match(/<div>(.*?)\s»\s(.*?)\s(?:<br><br>|<\/div>)/g);
		

		// if (results) {
		// 	for (let result of results) {
		// 		result = result.substring(result.indexOf(">")+1, result.lastIndexOf("<"));
		// 		let style_match = result.match(/(style=").*?"/g);
		// 		if (style_match != null)
		// 			result = result.replaceAll(style_match[0], "");
				
				
		// 		for (let line of result.split("<br>")) {
		// 			let card = line.split("»");
		// 			if (card && card.length == 2) {
		// 				let front = `<div>${card[0].trim()}</div>`;
		// 				let back = `<div>${card[1].trim()}</div>`;
		// 				newCards.push({ front, back });
		// 			}
		// 		}
		// 	}
		// }


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

		invoke("delete_card", {"cardId": card.card.id})

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

	let newPerDay = "8"
	async function createDeck() {
		// entry_deadline_date, deadline_complete
		if (await checkDeadlinePast()) return

		
		if (newName == "" || deadlineContents.some((x) => x.deck_name === newName)) return

		let md = {
			entry_type: "deck",
			deadline_date: null,
			study_intensity: null,
			new_per_day: parseInt(newPerDay)
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

	function stripHtml(content: string): string {
		return content.replace(/<\/?[^>]+(>|$)/g, '')
	}
	function toggleShowexplanationanation(card: CardDisplay) {
		card.show_explanation = !card.show_explanation;
		card_gallery = card_gallery
	}


	let chatMessages: ChatCompletionRequestMessage[] = [];
	let explanationMode = true
	async function getExplanation(card: CardDisplay) {
		if (!explanationMode)
			return

		const apiKey = await invoke("get_api_key")
		if (!apiKey) {
			card.card.explanation = "INVALID API KEY: RESUBMIT AND TRY AGAIN"
			return
		}

		card.loading = true;
		// process front and back
		let query = `I want you to act as Mnemonic Tutor. I will provide a flashcard, and you will explain it like I'm five years old. This could include providing step-by-step instructions for answering the question, suggesting mnemonics to remember it. NEVER address me directly and DON'T say "kiddo". Just go into the concise explanation. My first card is FRONT: "${stripHtml(card.card.front)}" BACK: "${stripHtml(card.card.back)} MNEMONIC TUTOR EXPLANATION: "`

		chatMessages = [{ role: 'user', content: query }]

		const systemPrompt = "Introducing Mnemonic Tutor designed to help students understand facts by explaining like I'm five. This AI-powered system limits its responses to three sentences and explains concepts in the simplest possible manner ONLY through intuitive examples."

		const unlistenChatGPT = await appWindow.listen(
			`CHATGPT_RESPONSE_${card.card.id}`,
			({ event, payload }: { event: string; payload: ResponseObjectType }) => {
				try {
					if (payload.choices[0].finish_reason === "stop") {
						chatMessages = []
						card.loading = false
						updateCard(card.card) // save explanation
						card_gallery = card_gallery

						unlistenChatGPT()
						return;
					}

					const delta = payload.choices[0].delta.content;

					if (delta) {
						card.card.explanation = card.card.explanation + delta;
						card_gallery = card_gallery

					}
				} catch (err) {
					loadingSuggestions = false
					unlistenChatGPT()

					handleError(err)
				}

			}
		);

		invoke('send_gpt_request', {
			apiKey,
			messages: [query],
			systemPrompt: systemPrompt,
			maxTokens: 1000,
			window: appWindow,
			cardId: card.card.id
		 });

	}


	import { appWindow } from '@tauri-apps/api/window';
	type Delta = {
		content: string | null;
		role: string | null;
	};

	type Choice = {
		delta: Delta;
		finish_reason: string | null;
		index: number;
	};

	type ResponseObjectType = {
		choices: Choice[];
	};


	let loadingSuggestions = false
	let suggestionMessages: ChatCompletionRequestMessage[] = [];
	async function getSuggestions() {
		if (panel.textfield == "")
			return;
		
		let text = panel.textfield.replaceAll("</div><div>", "</div>\n<div>") 
		text = stripHtml(text)
		text = text.replaceAll("•", "");
		const num_questions = text.split("?").length - 1
		
		text = `I want you to act as Card Creator. Card Creator will create cards (question-answer pairs) from the provided text. It separates cards with "<br><br>" after each answer. Restrictions: (1) question and answer are separated with ">>" (2) cards are separated with "<br><br>" (3) rephrase question if convoluted or ungrammatical (4) answer is ONE sentence (5) rephrase provided answer if given (6) create ${num_questions} card(s), each corresponding to a question in the notes (7) each line should be in the format question >> answer with no other information (8) avoid complete sentence outputs (9) do not number cards (10) cards have format {front of card >> back of card <br><br>}. Provided text: ${text} Card Creator created cards: `
		panel.textfield = ''
		loadingSuggestions = true;

		const apiKey = await invoke("get_api_key")
		if (!apiKey) {
			panel.textfield = "=== INVALID API KEY: RESUBMIT AND TRY AGAIN ===        " + panel.textfield
			return
		}

		suggestionMessages = [...suggestionMessages, { role: 'user', content: text }]
		// const systemPrompt = `Introducing the Card Creator, which can create flashcards from notes. It separates each card with "<br><br>", and it separates front and back of each card with ">>". Each card is a question-answer pair. If both the front and back of the flashcard is provided, AI Synthesizer only corrects grammar and simplifies prose. If only the front is provided, it gives a VERY concise back. Card Creator is a most powerful tool for creating flashcards from notes containing questions`
		const systemPrompt = `AI Synthesizer is designed to be able to create flashcards from a text of questions. It takes the same question and provides an answer if there is none. If there is an answer, it adapts the answer. It creates a flashcard only based on a question in the notes. I Synthesizer is a most powerful tool designed to create cards by understanding questions and providing good answers.`

		const unlistenChatGPT = await appWindow.listen(
			'CHATGPT_RESPONSE',
			({ event, payload }: { event: string; payload: ResponseObjectType }) => {
				try {
					if (payload.choices[0].finish_reason === "stop") {
						suggestionMessages = []
						panel.textfield = panel.textfield.replaceAll(">>", " » ")
						panel.textfield = panel.textfield.replaceAll("<br><br>", "<br></div><div>")
						loadingSuggestions = false

						unlistenChatGPT()
						return;
					}

					const delta = payload.choices[0].delta.content;
					if (delta) {
						panel.textfield = panel.textfield + delta;
					}

				} catch (err) {
					loadingSuggestions = false
					unlistenChatGPT()

					handleError(err)
				}

			}
		);

		invoke('send_gpt_request', {
			apiKey,
			messages: suggestionMessages.map((message) => message.content),
			systemPrompt: systemPrompt,
			maxTokens: 2000,
			window: appWindow 
		 });

			
	}

	function handleError<T>(err: T) {
		console.error(err)
	}

</script>

	
<div class="{isDarkMode ? "dark" : ""} bg-black h-full m-0 p-0">
	<div class="min-h-screen h-full bg-offwhite dark:bg-offblack">
		<div class="mb-5">
		
		<div class="flex justify-start space-x-5 mx-10">
			
			<!-- home button -->
			<a href="/" id="home-button" class="outline-columbia  ring-columbia mt-1 focus:outline-none focus:ring duration-75 rounded-md">
				<div class="fled justify-evenly ring-columbia  focus:outline-none focus:ring duration-75 rounded-md">
					<svg 
						fill="highlight" class="fill-columbia-dark dark:fill-columbia opacity-80 dark:opacity-100 float-left h-7 w-7 cursor-pointer outline-columbia focus:outline outline-4 outline-offset-2 " 
						viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path clip-rule="evenodd" fill-rule="evenodd" d="M9.293 2.293a1 1 0 011.414 0l7 7A1 1 0 0117 11h-1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-3a1 1 0 00-1-1H9a1 1 0 00-1 1v3a1 1 0 01-1 1H5a1 1 0 01-1-1v-6H3a1 1 0 01-.707-1.707l7-7z"></path>
					</svg>
				</div>
			</a>

			{#if !deadline_is_complete}
				<div class="bg-offwhite z-50 dark:bg-offblack" >
					<button on:click={toggleCreateDeckTray} class="h-9 w-9 z-50 dark:bg-offblack ring-columbia mt-1 focus:outline-none focus:ring duration-75 rounded-md">
						<Hint placement="bottom" text="Create Deck">
							<svg 
								class="fill-columbia-dark dark:fill-columbia opacity-80 dark:opacity-100 bg-inherit cursor-pointer h-7 w-8 outline-columbia focus:outline outline-4 outline-offset-2 " 
								xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" >
								<path fill-rule="evenodd" d="M5.625 1.5H9a3.75 3.75 0 013.75 3.75v1.875c0 1.036.84 1.875 1.875 1.875H16.5a3.75 3.75 0 013.75 3.75v7.875c0 1.035-.84 1.875-1.875 1.875H5.625a1.875 1.875 0 01-1.875-1.875V3.375c0-1.036.84-1.875 1.875-1.875zM12.75 12a.75.75 0 00-1.5 0v2.25H9a.75.75 0 000 1.5h2.25V18a.75.75 0 001.5 0v-2.25H15a.75.75 0 000-1.5h-2.25V12z" clip-rule="evenodd" />
								<path d="M14.25 5.25a5.23 5.23 0 00-1.279-3.434 9.768 9.768 0 016.963 6.963A5.23 5.23 0 0016.5 7.5h-1.875a.375.375 0 01-.375-.375V5.25z" />
							</svg>
						</Hint>
					</button>
				</div>
			{/if}

			<div class="bg-offwhite z-50 dark:bg-offblack">
				{#if !panel.display_textfield}
					<!-- front-back -->
					<button 
						class="h-9 w-9 ring-columbia mt-1  focus:outline-none focus:ring duration-75 rounded-md"
						on:click={toggleTextfield}>
						<Hint placement="bottom" text="AI Editor">
							<svg 
								class="stroke-columbia-dark dark:stroke-columbia  rounded-lg bg-inherit cursor-pointer h-7 w-7 outline-columbia focus:outline outline-4 outline-offset-2 " 
								fill="none" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
								<path d="M5.127 3.502L5.25 3.5h9.5c.041 0 .082 0 .123.002A2.251 2.251 0 0012.75 2h-5.5a2.25 2.25 0 00-2.123 1.502zM1 10.25A2.25 2.25 0 013.25 8h13.5A2.25 2.25 0 0119 10.25v5.5A2.25 2.25 0 0116.75 18H3.25A2.25 2.25 0 011 15.75v-5.5zM3.25 6.5c-.04 0-.082 0-.123.002A2.25 2.25 0 015.25 5h9.5c.98 0 1.814.627 2.123 1.502a3.819 3.819 0 00-.123-.002H3.25z"></path>
							</svg>
						</Hint>
					</button>

					
				{:else}
					<!-- textfield -->
					<button on:click={toggleTextfield} class="h-9 w-9 z-50 dark:bg-offblack ring-columbia mt-1 focus:outline-none focus:ring duration-75 rounded-md">
						<Hint placement="bottom" text="Flashcard Editor">
							<svg 
								class="fill-columbia-dark dark:fill-columbia opacity-80 dark:opacity-100 rounded-lg bg-inherit cursor-pointer h-7 w-7 outline-columbia focus:outline outline-4 outline-offset-2 " 
								fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
								<path d="M5.127 3.502L5.25 3.5h9.5c.041 0 .082 0 .123.002A2.251 2.251 0 0012.75 2h-5.5a2.25 2.25 0 00-2.123 1.502zM1 10.25A2.25 2.25 0 013.25 8h13.5A2.25 2.25 0 0119 10.25v5.5A2.25 2.25 0 0116.75 18H3.25A2.25 2.25 0 011 15.75v-5.5zM3.25 6.5c-.04 0-.082 0-.123.002A2.25 2.25 0 015.25 5h9.5c.98 0 1.814.627 2.123 1.502a3.819 3.819 0 00-.123-.002H3.25z"></path>
							</svg>
						</Hint>
					</button>
				{/if}
			</div>


			<div class="bg-offwhite z-50 dark:bg-offblack mt-1">
				{#if !explanationMode}
					<Hint placement="bottom" text="Generate Explanations">
						<button on:click={() => explanationMode = true} id="show-explanation-button" class="focus:outline-none focus:ring-2 ring-columbia rounded-sm p-1 mr-3 ">
							<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 stroke-columbia-dark dark:stroke-columbia">
								<path stroke-linecap="round" stroke-linejoin="round" d="M12 18v-5.25m0 0a6.01 6.01 0 001.5-.189m-1.5.189a6.01 6.01 0 01-1.5-.189m3.75 7.478a12.06 12.06 0 01-4.5 0m3.75 2.383a14.406 14.406 0 01-3 0M14.25 18v-.192c0-.983.658-1.823 1.508-2.316a7.5 7.5 0 10-7.517 0c.85.493 1.509 1.333 1.509 2.316V18" />
							</svg>
						</button>
					</Hint>
				{:else}

					<Hint placement="bottom" text="Generate Explanations">
						<button on:click={() => explanationMode = false} id="show-explanation-button" class="focus:outline-none focus:ring-2 ring-columbia rounded-sm p-1 mr-3 ">
							<svg xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 fill-columbia-dark dark:fill-columbia stroke-columbia-dark dark:stroke-columbia">
								<path stroke-linecap="round" stroke-linejoin="round" d="M12 18v-5.25m0 0a6.01 6.01 0 001.5-.189m-1.5.189a6.01 6.01 0 01-1.5-.189m3.75 7.478a12.06 12.06 0 01-4.5 0m3.75 2.383a14.406 14.406 0 01-3 0M14.25 18v-.192c0-.983.658-1.823 1.508-2.316a7.5 7.5 0 10-7.517 0c.85.493 1.509 1.333 1.509 2.316V18" />
							</svg>
						</button>
					</Hint>
				{/if}
			</div>

			  

		</div>
		{#if createDeckTrayOpen}
			<form class="w-1/3 absolute left-28 py-1 z-50 bg-offwhite dark:bg-offblack dark:text-whitetext"
					on:submit={createDeck} use:clickOutside on:click_outside={toggleCreateDeckTray}>
					<!-- disregard the above warning -->
				<div class="border p-4 rounded-lg rounded-tl-sm border-columbia py-2 grid grid-rows-2 grid-cols-3 gap-2">
					<!-- name for Create -->
					<input type="text" use:focus placeholder="Enter Name" bind:value={newName} class="h-8 col-span-2 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-3 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring-2 duration-75"/>
					<!-- <label for="quantity">Enter a number:</label> -->
					<Hint placement="top" text="Set New Cards Per Day">
						<input bind:value={newPerDay} type="number" id="quantity" name="quantity" min="1" max="100" step="1"
							class="h-8 col-span-1 appearance-none hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-3 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring-2 duration-75">
					</Hint>

					<button type="submit" class="h-8 col-span-3 text-sm hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia  focus:outline-none focus:ring-2 duration-75">
						Create Deck	
					</button>
				</div>
			</form>


		{/if}
		
	
	<div class="h-5"></div>

	 <!-- central panel -->
	<div class="mt-4 h-full flex flex-col justify-center items-center">
		<!-- card fields -->
		<div class="card flex flex-col h-full border-slate-700 dark:border-none rounded-lg text-blacktext w-full sm:w-[600px] md:w-[650px] lg:w-[700px] dark:text-whitetext">

			{#key clearEditorToggle}
			{#if !panel.display_textfield}
				
				<!-- front field -->
				<div class="h-full max-h-80 mx-2 mb-1 border-l rounded-md border-columbia-dark dark:border-columbia" >         
					<div class="h-full p-1 rounded-lg ">
						<TextfieldEditor bind:content={panel.front} autofocus={true}/>
					</div>
				</div>          

				<!-- back field -->
				<div class="h-full max-h-80 mx-2 border-l rounded-md mb-1 border-columbia-dark dark:border-columbia" >         
					<div class="h-full p-1 rounded-lg ">
						<TextfieldEditor bind:content={panel.back} />
					</div>
				</div>          
					

			{:else}
				<div class="h-full m-3 mb-2 text-inherit" >         
					<div class="h-full p-1 rounded-lg border-l border-columbia-dark dark:border-columbia">
						{#key loadingSuggestions}
							<TextfieldEditor bind:content={panel.textfield} is_textfield={true} loading={loadingSuggestions} />
						{/key}
					</div>
				</div>
			{/if}
			{/key}
			
			<!-- make this one bar -->
			<div class="flex items-center justify-center top-[450px]">     
			{#if deadlineContents && deadlineContents.length > 0 && !deadline_is_complete}

				<!-- suggest -->
				{#if panel.display_textfield}
					<button 
						on:click={getSuggestions} 
						class="h-8 w-1/6 relative z-50 inline-flex items-center stroke-blacktext fill-columbia justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all border-y border-l border-gray-200 rounded-bl-lg cursor-pointer group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75">
						<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-6 h-6 fill-columbia-dark dark:fill-columbia stroke-columbia dark:stroke-blacktext stroke-0">
							<path fill-rule="evenodd" d="M9 4.5a.75.75 0 01.721.544l.813 2.846a3.75 3.75 0 002.576 2.576l2.846.813a.75.75 0 010 1.442l-2.846.813a3.75 3.75 0 00-2.576 2.576l-.813 2.846a.75.75 0 01-1.442 0l-.813-2.846a3.75 3.75 0 00-2.576-2.576l-2.846-.813a.75.75 0 010-1.442l2.846-.813A3.75 3.75 0 007.466 7.89l.813-2.846A.75.75 0 019 4.5zM18 1.5a.75.75 0 01.728.568l.258 1.036c.236.94.97 1.674 1.91 1.91l1.036.258a.75.75 0 010 1.456l-1.036.258c-.94.236-1.674.97-1.91 1.91l-.258 1.036a.75.75 0 01-1.456 0l-.258-1.036a2.625 2.625 0 00-1.91-1.91l-1.036-.258a.75.75 0 010-1.456l1.036-.258a2.625 2.625 0 001.91-1.91l.258-1.036A.75.75 0 0118 1.5zM16.5 15a.75.75 0 01.712.513l.394 1.183c.15.447.5.799.948.948l1.183.395a.75.75 0 010 1.422l-1.183.395c-.447.15-.799.5-.948.948l-.395 1.183a.75.75 0 01-1.422 0l-.395-1.183a1.5 1.5 0 00-.948-.948l-1.183-.395a.75.75 0 010-1.422l1.183-.395c.447-.15.799-.5.948-.948l.395-1.183A.75.75 0 0116.5 15z" clip-rule="evenodd" />
						</svg>
					</button>

				{/if}

				<!-- Create -->
				<button 
					on:click={panel.display_textfield ? createCardTextfield : createCardFrontBack} 
					class="h-8 w-1/3 relative z-40 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all border-y border-l border-gray-200 {!panel.display_textfield ? "rounded-bl-lg" : ""} cursor-pointer group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75">
					<svg class="fill-columbia-dark dark:fill-columbia stroke-columbia flex-none h-6 w-6 outline-columbia focus:outline outline-4 outline-offset-2 "
						stroke="currentColor" stroke-width="2" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15"></path>
					</svg>
				</button>


				<!-- select deck -->
				<div style="text-align-last:center;" class="border-y px-3 py-1 {panel.display_textfield ? "w-3/4" : "w-1/2"} h-8 relative z-30 outline-columbia focus:outline outline-4 outline-offset-2 overflow-hidden font-bold text-gray-500 transition-all duration-200 border-gray-200 cursor-pointer group ease border-columbia  bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white">
					<select bind:value={panel.selected_deck}
						class="text-columbia relative appearance-none w-full h-full outline-columbia focus:outline outline-4 outline-offset-2 justify-center z-30 inline-flex overflow-hidden font-bold text-gray-500 border-gray-200 cursor-pointer group ease border-columbia bg-gradient-to-b from-offwhite dark:from-offblack dark:to-offblack dark:border-x dark:rounded-none to-gray-50 hover:from-gray-50 hover:to-white active:to-white transition-all duration-100 {panel.display_textfield ?? "border-x border-columbia"}"
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
						<svg class="fill-columbia-dark dark:fill-columbia flex-none h-7 w-7 cursor-pointer outline-columbia focus:outline outline-4 outline-offset-2 "
							fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
							<path clip-rule="evenodd" fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"></path>
						</svg>
					</button>
				{:else}
					<button 
						class="h-8 w-1/4 relative z-30 text-platinum inline-flex items-center justify-center px-8 py-3 pt-4 overflow-hidden font-bold text-gray-500 transition-all border-y border-r border-gray-200 rounded-br-lg cursor-default group ease border-columbia  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-75"
					>
						<svg class="fill-columbia-dark dark:fill-columbia opacity-50 flex-none h-8 w-8 outline-columbia focus:outline outline-4 outline-offset-2"
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
							class="fill-columbia-dark dark:fill-columbia  bg-inherit cursor-pointer h-6 w-8 outline-columbia focus:outline outline-4 outline-offset-2 " 
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
							{#if deadlineContents && deadlineContents.length > 1}
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
								class="pl-2 h-8 placeholder:opacity-50 placeholder:text-center border-y border-r rounded-r-lg w-full border-columbia py-2 pr-10 focus:outline-none"
								placeholder="Filter Cards"/>

							{:else}
							<input type="text"
								bind:value={panel.prompt}
								on:change={filterCards}
								class="rounded-l-lg placeholder:opacity-50 placeholder:text-center border-l pl-4 h-8 border-y border-r rounded-r-lg w-full border-columbia py-2 pr-10 focus:outline-none"
								placeholder="Filter Cards"/>

							{/if}

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
		
		<div class="flex flex-col gap-3 mt-2">   
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
							<div class="w-1/2 flex-1 border-x border-t { !card.show_explanation ?  "border-b" : "rounded-b-none" } rounded-lg border-dotted border-columbia">
								<div on:focusout={() => updateCard(card.card)} class="w-full px-1 py-2 rounded-lg">
									<TextfieldEditor bind:content={card.card.front} is_gallery={true}/>
								</div>
							</div>						
							

							<div class="w-1/2 flex-1 border-r border-t { !card.show_explanation ?  "border-b" : "rounded-b-none" } rounded-lg border-dotted border-columbia">
								<div on:focusout={() => updateCard(card.card)} class="w-full  px-2 py-2 rounded-lg">
									<TextfieldEditor bind:content={card.card.back} is_gallery={true}/>
								</div>	
							</div>
						</div>
						

						{#if card.show_explanation}
							<div class="">
								<div on:focusout={() => updateCard(card.card)} class="w-full opacity-90 border-x border-y rounded-b-lg rounded-tr-sm border-dotted border-columbia border-spacing-4 px-2 py-2">
									{#key card.loading}
										<TextfieldEditor bind:content={card.card.explanation} is_expl={true} loading={card.loading}/>
									{/key}
								</div>					
							</div>
						{/if}
						

					</div>
					<div class="flex flex-col justify-between">
						<span class="cursor-pointer hover:opacity-100 pt-3 right-1 dark:opacity-50 text-md relative text-columbia" on:click={() => deleteCard(card)} on:keydown={() => deleteCard(card)}> 
							<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
								<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
							  </svg>
							  
						</span>

						<span class=" cursor-pointer hover:opacity-100 pb-1 right-2 pl-1 bottom-2 dark:opacity-50 relative text-md text-columbia" on:click={() => toggleShowexplanationanation(card)} on:keydown={() => toggleShowexplanationanation(card)}> 
							{#if !card.show_explanation}
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

	.scroll_pad {
		height: calc(100vh + 200px);
	}



</style> 
