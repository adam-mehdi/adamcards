<script lang="ts">
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';
	import { quintOut } from 'svelte/easing';                                      
    import { crossfade } from 'svelte/transition';                                 
    import { flip } from 'svelte/animate';                                         
	import { onMount, onDestroy } from 'svelte';
	import configStore from '$lib/stores/configStore'
	import Hint from 'svelte-hint';
	import { fade } from 'svelte/transition';
	import TextfieldEditor from '$lib/TextfieldEditor.svelte'
	import ChatTextfieldEditor from '$lib/ChatTextfieldEditor.svelte'

  	import type { ChatCompletionRequestMessage } from 'openai';
	import { SSE } from 'sse.js';



	// write buf_size - min_history every time
	let isDarkMode = $configStore.is_dark_mode;

	// onMount(() => {
	// 	disableScrolling();
	// });

	// function disableScrolling() {
	// 	document.body.style.overflow = 'hidden';
	// }

	
	// FrontendCard contains properties that can be edited in the frontend
	interface Card {
		id: number,
		front: string,
		back: string,
		explanation: string
	}

	interface ReviewCard {
		stack_before: string, // can be "new" | "review" | "done"
		deck_name: string,
		card: Card
	}
	
	// review Quota
	interface Quota {
		new_left: number,
		review_left: number,
		num_progressed: number
	}

	interface AnimatedCardStacks {
		new: number[],
		review: number[],
		done: number[],
		studying: number[]
	}

	interface NextIntervals {
		r1: number;
		r2: number;
		r3: number;
		r4: number;
		r5: number;
	}

	

	/**
	 * State for review session
	 */
	let stacks: AnimatedCardStacks;
	// let num_progressed_init: number // make num_progressed stack reset between sessions
	let cardIsRevealed: boolean = false;   // back field is revealed
	let sessionStarted: boolean = false; 	 // false if session has not started yet
	let sessionFinished: boolean = false;   // review session is completed
	let userAnswer: string = '';
	// let id2idx: Map<number, Card>,
	let currCard: ReviewCard;
	const deadlineId: number = parseInt($page.params.entry);
	let isAnki: boolean;
	let nextIntervals: NextIntervals = { r1: 0, r2: 0, r3: 0, r4: 0, r5: 0 };

	

	// initializes frontend and backend state
	async function initState() {
		isAnki = await invoke("get_is_anki_frontend", { deadlineId });
		let quota: Quota = await invoke('init_review_session', { deadlineId });


		let range = (n: number) => Array.from(Array(n).keys());
		stacks = {
			new: range(quota.new_left),
			review: range(quota.review_left).map(x => x + quota.new_left),
			done: range(quota.num_progressed).map(x => x + quota.review_left + quota.new_left),
			studying: [],
		}
	}
	initState()


	async function getNextCard() {
		cardIsRevealed = false;
		loading = false
		aiMessage = ""
		messages = []
		chatMessages = []

		let card: ReviewCard | null = await invoke("get_next_card", {})
		
		// if no card was returned, session is finished
		if (!card) {
			sessionFinished = true;
			return
		}

		currCard = card

		// remove card from proper stack and decrease quota
		let stack = card.stack_before == "new" ? stacks.new : stacks.review;		
		if (stack.length == 0) {
			console.error("taking card out of an empty stack");
		}

		const study_card: number = stack.pop()!;
		stacks.studying.push(study_card);

		// re-render the DOM
		showExplanation = false;
		userAnswer = '';
		resetAnswerBar = !resetAnswerBar;

		stacks = stacks;
		// console.log(await invoke("print_cards", {deadlineId}))

		nextIntervals = await invoke("get_next_intervals", { "cardId": currCard.card.id });
	}

	function updateCard(card: Card) {
		invoke("update_card", { card })
	}

	let okayCanFire = false;
	async function handleResponse(score: number) {
		// prevent "Okay" from automatically firing		
		if (score == 3 && !okayCanFire) {
			okayCanFire = true;
			return;
		}
		okayCanFire = false;

		score = isAnki ? score : score - 3;

		let stack_after: string = await invoke("record_response", { score, userAnswer, "card": currCard })

		// put the card in that stack
		let new_stack = get_new_stack(stack_after);

		const studying_id = stacks.studying.pop()!;
		let insert_idx = stack_after == "done" ? new_stack.length : Math.floor(new_stack.length / 2);
		new_stack.splice(insert_idx, 0, studying_id);
		// id2idx.set(buf_card.card.id, study_idx);

		if (new_stack.length == 1 && (score < 2 && isAnki) || score < 1) {
			cardIsRevealed = false;
			stacks = stacks;
			await sleep(900); // hack to avoid frontend rendering bug
			
			stacks.studying.push(new_stack.pop()!)
			stacks = stacks
			return
		}

		stacks = stacks;
		getNextCard() 
		num_forward += 1;
	}

	function sleep(ms: number) {
		return new Promise(resolve => setTimeout(resolve, ms));
	}


	// undo getNextCard, and reverse quotas state
	interface CardResults {
		stack_after: string;
		user_answer: string;
		card: ReviewCard;
	}

	let num_forward = 0;
	let num_back = 0;
	async function getLastCard() {
		if (!sessionStarted || sessionFinished)
			return

		// return if no last card recorded
		let results: CardResults | null = await invoke("get_last_card", {})
		if (!results)
			return;

		// put card currently being studied into stack before
		let currCardStack = get_new_stack(currCard.stack_before);
		let studying: number = stacks.studying.pop()!;
		currCardStack.push(studying);
		stacks = stacks;

		// draw last card from stack it ended up in
		currCard = results.card
		let lastCardStack = get_new_stack(results.stack_after);
		let lastCardId = lastCardStack.pop()!;
		stacks.studying.push(lastCardId);
		stacks = stacks

		// show user answer again
		userAnswer = results.user_answer;
		stacks = stacks
		num_forward -= 1
		num_back += 1
		
	}

	async function undoGetLastCard() {
		if (!sessionStarted || sessionFinished)
			return

		let results: CardResults | null = await invoke("undo_get_last_card", {})
		if (!results) {
			getNextCard();
			return
		}
		
		// put card currently being studied into stack after
		let currCardStack = get_new_stack(results.stack_after);
		let studying: number = stacks.studying.pop()!;
		currCardStack.push(studying);
		stacks = stacks;


		// draw next card from stack before
		let nextCardStack = get_new_stack(results.card.stack_before)
		let nextCardId = nextCardStack.pop()!
		stacks.studying.push(nextCardId)
		stacks = stacks

		// show user answer again
		userAnswer = results.user_answer;
		stacks = stacks
		num_forward += 1
		num_back -= 1
	}

	// get stack to put the card into, where it will end up after a response
	function get_new_stack(stack: string): number[] {
		let new_stack;
		if (stack == "new") { 
			new_stack = stacks.new; 
		} else if (stack == "review") { 
			new_stack = stacks.review; 
		} else if (stack == "done") {
			new_stack = stacks.done;
		} else {
			console.error(stack, "not a stack");
			return [];
		}
		return new_stack;
	}


	function revealCard() {
		cardIsRevealed = true;
		stacks.studying = stacks.studying
	}


	function onKeyDown(e: KeyboardEvent) {

		// detect whether user is editing card front or back (if they have focus)
		const activeElement = document.activeElement;
		const front = document.getElementById('front');
		const back = document.getElementById('back');
		const bar = document.getElementById('user-answer-bar');
		const inst = document.getElementById('show-instruction-button');
		const expl = document.getElementById('show-explanation-button');
		// if user is editing card, don't trigger callbacks
		if (activeElement === front || activeElement === back || activeElement === inst 
			|| activeElement === expl || showExplanation) {
			// console.log(currCard.card.explanation)
			return;

		}

		// go back or forward through the buffer given user responses
		if (e.key == "<TODO")
			getLastCard();
		else if (e.key == ">TODO") 
			undoGetLastCard(); 
		
		// session starts or back is revealed if user presses enter
		if (e.key == "Enter") {
			if (!sessionStarted) {
				sessionStarted = true;
				getNextCard();
			} else {
				recordMessage();
				revealCard()
			}
		}

		if (activeElement != bar  && activeElement != front  && activeElement != back && cardIsRevealed) {
			if (e.key == "1") 	   handleResponse(1)
			else if (e.key == "2") handleResponse(2)
			else if (e.key == "3") handleResponse(3)
			else if (e.key == "4") handleResponse(4)
			else if (e.key == "5") handleResponse(5)
		}

		
	} 

			

	/**
	 * Code for animating cards
	 */

	// each number refers to one animated card on the screen
	// cards are moved on screen by moving them between arrays

	let windowHeight = 0;
	$: getStackHeight = () => windowHeight > 650 ? Math.floor(windowHeight / 30) : Math.floor(windowHeight / 55);

	const MOVE_DURATION = 50;
    const [send, receive] = crossfade({                                            
        fallback(node, params) {                                                   
            const style = getComputedStyle(node);                                  
            const transform = style.transform === 'none' ? '' : style.transform;
                                                                                   
            return {                                                               
                duration: 600,                                                     
                easing: quintOut,                                                  
                css: t => `                                                        
                    transform: ${transform} scale(${t});                           
                    opacity: ${t}                                                  
                `                                                                  
            };                                                                     
        }                                                                          
    });
	

	let showExplanation = false
	function toggleShowExplanation() {
		showExplanation = !showExplanation
	}

	onMount(() => {
		setTimeout(() => {
			sessionStarted = true
			getNextCard()
		}, 200);
	});

	interface Message {
		type: "user" | "assistant",
		content: string
	}
	let messages: Message[] = []
	let resetAnswerBar = false;
	let chatMessages: ChatCompletionRequestMessage[] = [];

	let loading = false
	function recordMessage() {
		loading = true
		const processedAnswer = stripHtml(userAnswer)
		if (processedAnswer.length == 0) {
			userAnswer = ''
			loading = false
			resetAnswerBar = !resetAnswerBar
			return
		}

		messages.push({ 
			type: "user", 
			content: processedAnswer
		})
		userAnswer = ''
		messages = messages
		resetAnswerBar = !resetAnswerBar

		getInstruction(processedAnswer)



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
	let aiMessage = ""
	async function getInstruction(processedAnswer: string, ) {
		// send request to chatGPT
		// let query = "QUESTION: " + stripHtml(currCard.card.front) + stripHtml(currCard.card.back) + " RESPONSE: " + processedAnswer

		const apiKey = await invoke("get_api_key")
		if (!apiKey) {
			aiMessage = "INVALID API KEY: RESUBMIT AND TRY AGAIN     " + aiMessage
			return
		}

		let query = messages.length === 1
			? `Now AI Instructor will evaluate my guess in at most two sentences. Restrictions: (1) explain if seems correct, partially correct or incorrect, defaulting to incorrect (2) address incorrect points by explaining how things actually are (3) a guess is correct if it says the same thing as the answer (4) don't congratulate (5) avoid restating the answer. Question: ${stripHtml(currCard.card.front)} True Answer: ${stripHtml(currCard.card.back)} My guess: ${stripHtml(userAnswer)}  AI Instructor comparing question and answer:`
			: `Write in small paragraphs, putting "<br><br>" between each paragraph. Respond to my prompt with elucidating examples. My prompt: "${processedAnswer}" Your explanation: `
		
		let systemPrompt = messages.length === 1
			? "AI Instructor is designed to be able to assess if the answer given to the card was accurate or on track, highlighting any mistakes. It avoids repeating what the card says. It is fun and kind. Every once in a while, it discretely slips in a joke. Keep it brief and concise, without apologies, and limit your response to two sentences. AI Instructor is a powerful tool to evaluate guesses against answer."
			: `AI Assistant is designed to give a broader understanding of a fact. Answer concisely and helpfully, with short paragraphs and "<br><br>" between each. Avoid repeating what has been said. AI Instructor is a powerful tool to explain new concepts.`

		chatMessages = [...chatMessages, { role: 'user', content: query }]

		const unlistenChatGPT = await appWindow.listen(
			'CHATGPT_RESPONSE',
			({ event, payload }: { event: string; payload: ResponseObjectType }) => {
				try {
					// do nothing if user got next card when ChatGPT was in the middle of responding
					if (messages.length == 0)
						return
						
					if (payload.choices[0].finish_reason === "stop") {
						loading = false

						chatMessages.push({
							"role": 'assistant', 
							"content": aiMessage
						})

						messages.push({
							type: "assistant",
							content: aiMessage
						})
						aiMessage = ""
						messages = messages
						let myDiv: any = document.getElementById("chatbox");

						setTimeout(() => {
							// const isAtBottom = myDiv.scrollHeight - myDiv.scrollTop === myDiv.clientHeight;

							// if (isAtBottom) {
							myDiv.scrollTop = myDiv.scrollHeight;
							// }
						}, 250);

						unlistenChatGPT()
						return;
					}

					if (messages.length > 0) {
						const delta = payload.choices[0].delta.content;
						if (delta) {
							aiMessage = aiMessage + delta
							let myDiv: any = document.getElementById("chatbox");
							myDiv.scrollTop = myDiv.scrollHeight;
						}
					}
				} catch (err) {
					loading = false
					unlistenChatGPT()

					handleError(err)
				}

			}
		);

		invoke('send_gpt_request', {
			apiKey,
			messages: chatMessages.map((message) => message.content),
			systemPrompt: systemPrompt,
			maxTokens: 2000,
			window: appWindow 
		 });
		
	}

	function handleError<T>(err: T) {
		console.error(err)
		messages = messages
		loading = false
	}

	function stripHtml(content: string): string {
		return content.replace(/<\/?[^>]+(>|$)/g, '')
	}

	async function deleteCard() {

		stacks.studying.pop()!;
		await invoke("delete_card", {"cardId": currCard.card.id})

		getNextCard()
		stacks = stacks
	}

</script>


<!-- Listen for keyboard events -->
<svelte:window on:keydown={onKeyDown} bind:innerHeight={windowHeight}/>
<!-- Home button -->
<div class={isDarkMode ? "dark overflow-none" : ""}>
<div class="bg-offwhite dark:bg-offblack min-h-screen text-blacktext {cardIsRevealed ?? 'left_navbar'} ">
	<div class="{windowHeight > 450 ? "h-5" : "h-2"}"></div>

	<!-- bar at top -->
		<div class="ml-8 h-18 w-6 ">
			<a href="/" class="ring-columbia focus:outline-none focus:ring duration-75">
				<div class="fled justify-evenly w-6 ">
					<svg fill="highlight" class="flex-none h-6 w-6 cursor-pointer ring-columbia focus:outline-none focus:ring duration-75 rounded-md" 
						viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path clip-rule="evenodd" fill-rule="evenodd" d="M9.293 2.293a1 1 0 011.414 0l7 7A1 1 0 0117 11h-1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-3a1 1 0 00-1-1H9a1 1 0 00-1 1v3a1 1 0 01-1 1H5a1 1 0 01-1-1v-6H3a1 1 0 01-.707-1.707l7-7z"></path>
					</svg>
				</div>
			</a>
		</div>


		<div class=" {windowHeight > 450 ? "space-y-8" : "space-y-4"}">

		{#if !sessionStarted && stacks}
			<!-- <h3 class="text-center font-bold text-columbia text-4xl">Welcome, </h3> -->
			<h3 class="text-center font-mono font-bold text-columbia text-xl">Godspeed!</h3>
			<!-- <h3 class="text-center font-serif font-semibold text-columbia text-md">With {stacks.new.length} new and {stacks.review.length} review cards, today's practice will take {(stacks.new.length * 12 + stacks.review.length * 5) / 60} minutes.</h3> -->


		{:else if sessionStarted} <!-- show card field if session has started, but not finished -->
			{#if !sessionFinished}

				<!-- {#key card_drawn}  -->
				<div class='mx-auto {messages.length == 0 || !cardIsRevealed ? "mt-0" : "-mt-0" } '>
					{#each stacks.studying as id (id)}

						<div class="card-container z-50 w-1/2 mx-auto"
							in:receive="{{key: id}}"
							out:send="{{key: id}}"
							animate:flip="{{duration: 50}}">

								<div class="{!isDarkMode ? "card" : ""} w-full dark:border-x dark:border-y dark:border-opacity-50 border-columbia border-opacity-50 flex flex-col h-full rounded-lg text-blacktext dark:bg-slate-700 dark:text-offwhite">
			
									<div class="flex-row flex justify-between">
										<div class="opacity-40 font-serif ml-4 mt-1">
											<Hint placement="top" text="Deck Name">
												{currCard.deck_name}
											</Hint>
										</div>

										<div class="mr-2 mt-2 opacity-70">
											<Hint placement="top" text="Delete Card">
												<span class="cursor-pointer hover:opacity-100 pt-3 dark:opacity-30 text-md text-columbia" on:click={() => deleteCard()} on:keydown={() => deleteCard()}> 
													<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
														<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
													</svg>
												</span>
											</Hint>
										</div>
									</div>




									<!-- front field -->
									<div on:focusout={() => updateCard(currCard.card)} class="w-[520px] lg:w-[700px] mx-8 {messages.length == 0 ? "mb-3 mt-1" : "my-1"} text-inherit dark:bg-slate-700 dark:text-columbia p-2 rounded-lg" >    
										<TextfieldEditor bind:content={currCard.card.front} is_reviewfront={true} is_large={true} />
									</div>          
									
									<!-- rule separating front and back fields -->
									<div class="border-t border-1 border-opacity-50 border-columbia" />   

									<!-- back field -->
									{#if !cardIsRevealed}
										<div class="mx-8 my-6 text-inherit"></div>          
									{:else}
										<div on:focusout={() => updateCard(currCard.card)} class="mx-8 mt-4 text-inherit dark:bg-slate-700 p-2 rounded-lg dark:text-columbia" transition:fade="{{duration: 150 }}" >         
											<TextfieldEditor bind:content={currCard.card.back} is_reviewback={true} is_large={true}/>
										</div>          
									{/if}
										

									{#if cardIsRevealed}
									<!-- answer bar -->

										<!-- number of days -->
										<div class="flex font-serif items-center justify-center">     
													
											<div class="h-5 relative {isAnki ? "w-1/5" : "w-1/3"} z-30 inline-flex items-center justify-center px-8 py-1 text-sm overflow-hidden font-light opacity-50 text-gray-500 rounded-bl-lg cursor-pointer group ease  outline-columbia focus:outline pointer-text ring-columbia focus:outline-none focus:ring duration-0">
												{nextIntervals.r1}d
											</div>
										
											<div class="h-5 relative {isAnki ? "w-1/5" : "w-1/3"} z-30 inline-flex items-center justify-center px-8 py-1 text-sm overflow-hidden font-light opacity-50 text-gray-500 rounded-bl-lg cursor-pointer group ease  outline-columbia focus:outline pointer-text ring-columbia focus:outline-none focus:ring duration-0">
												{nextIntervals.r2}d
											</div>
													
											<div class="h-5 relative {isAnki ? "w-1/5" : "w-1/3"} z-30 inline-flex items-center justify-center px-8 py-1 text-sm overflow-hidden font-light opacity-50 text-gray-500 rounded-bl-lg cursor-pointer group ease  outline-columbia focus:outline pointer-text ring-columbia focus:outline-none focus:ring duration-0">
												{nextIntervals.r3}d
											</div>

											<div class="h-5 relative {isAnki ? "w-1/5" : "w-1/3"} z-30 inline-flex items-center justify-center px-8 py-1 text-sm overflow-hidden font-light opacity-50 text-gray-500 rounded-bl-lg cursor-pointer group ease  outline-columbia focus:outline pointer-text ring-columbia focus:outline-none focus:ring duration-0">
												{nextIntervals.r4}d
											</div>

											<div class="h-5 relative {isAnki ? "w-1/5" : "w-1/3"} z-30 inline-flex items-center justify-center px-8 py-1 text-sm overflow-hidden font-light opacity-50 text-gray-500 rounded-bl-lg cursor-pointer group ease  outline-columbia focus:outline pointer-text ring-columbia focus:outline-none focus:ring duration-0">
												{nextIntervals.r5}d
											</div>
										</div>

										<div class="flex font-mono items-center justify-center">     
													
											<button 
												on:click={() => handleResponse(1)}
												class="h-5 relative {isAnki ? "w-1/5" : "w-1/3"} relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 border-y border-l border-columbia rounded-bl-lg cursor-pointer group ease  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-0">
												1
											</button>
											
										
											<button 
												on:click={() => handleResponse(2)}
												class="h-5 {isAnki ? "w-1/5" : "w-1/3"} relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 border-y border-l border-columbia {isAnki ? "" : "rounded-bl-none" } cursor-pointer group ease  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-0">
												2

											</button>

											<button
												on:click={() => handleResponse(3)}
												on:keypress={() => handleResponse(3)}
												class="h-5 {isAnki ? "w-1/5" : "w-1/3"} relative z-40 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 border-y border-x border-columbia cursor-pointer group ease  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-0">
												3 
											</button>      
											<!-- no more autofocus on Okay -->

											{#if isAnki}
												<button	
													class="h-5 {isAnki ? "w-1/5" : "w-1/3"} relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 border-y border-r border-l border-columbia {isAnki ? "" : "rounded-br-lg" } cursor-pointer group ease  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white  ring-columbia focus:outline-none focus:ring duration-0"
													on:click={() => handleResponse(4)} >
													4
												</button>    

												<button 
													on:click={() => handleResponse(5)}
													class="h-5 {isAnki ? "w-1/5" : "w-1/3"} relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 border-y border-l border-columbia rounded-br-lg cursor-pointer group ease  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white ring-columbia focus:outline-none focus:ring duration-0">
													5
												</button>
											{/if}
										</div>
										

									{/if}
								</div>


								<!-- answer bar -->
								<form class="w-full outline-none">
									<div class="w-full flex flex-wrap items-stretch relative mb-3">
										<div class = "w-5/6 h-full mx-auto dark:text-whitetext rounded-md border-1 pl-3 -pr-16 pt-1 pb-2 outline-none cursor-text">
											{#if !showExplanation}
												<div id="user-answer-bar" class=" border h-full overflow-scroll rounded-lg mr-2 focus:outline-none duration-75">
													<!-- history -->
													<div id="chatbox" class="max-h-80 overflow-y-scroll scroll-my-12 flex flex-col z-50">
														{#each messages as message, index}
															{#if loading && index % 2 == 1 && index == messages.length - 1}
																<p class="{index == messages.length - 1 ?? "mb-1"} ring-columbia font-light bg-platinum dark:bg-slate-700 inline-block px-3 rounded-md p-2 m-2 mr-8 cursor-text text-light transition-opacity duration-100 text-md">
																	{message.content}	
																</p>
															{:else}
																<ChatTextfieldEditor index={index} content={message.content} />
															{/if}
														{/each}
														{#if aiMessage.length > 0}
															<div class="font-light inline-block text-md px-4 rounded-md bg-platinum dark:bg-slate-700 p-2 m-2 mr-8 cursor-text focus-within:ring-2  ring-columbia transition-opacity duration-100">
																{@html aiMessage}
															</div>
														{/if}
													</div>
													<div>
														{#key resetAnswerBar}
															{#if !loading}
																<!-- {#key loading} -->
																<div class="pl-2">
																	<TextfieldEditor bind:content={userAnswer} autofocus={true} is_answerbar={true} is_useranswer={true}/>
																</div>
																<!-- {/key} -->
															{/if}
														{/key}
													</div>
												</div>
											{:else}
												<div class="bg-platinum dark:bg-slate-700 border h-full rounded-lg mr-2 ring-columbia focus:outline-none focus:ring-2 duration-75">
													<TextfieldEditor bind:content={currCard.card.explanation} is_useranswer={true}/>
												</div>
											{/if}
										</div>

										<span class="right-1/4 absolute inset-y-0 flex items-end mb-3 ml-1 my-2 -mr-24 lg:-mr-32 -pr-1 lg:pr-1 z-50">
											{#if !showExplanation}
												<Hint placement="bottom" text="Show Explanation">
													<button on:click={toggleShowExplanation} id="show-explanation-button" class="focus:outline-none focus:ring-2 ring-columbia rounded-sm p-1 mr-3 ">
														<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 dark:invert">
															<path stroke-linecap="round" stroke-linejoin="round" d="M12 18v-5.25m0 0a6.01 6.01 0 001.5-.189m-1.5.189a6.01 6.01 0 01-1.5-.189m3.75 7.478a12.06 12.06 0 01-4.5 0m3.75 2.383a14.406 14.406 0 01-3 0M14.25 18v-.192c0-.983.658-1.823 1.508-2.316a7.5 7.5 0 10-7.517 0c.85.493 1.509 1.333 1.509 2.316V18" />
														</svg>
													</button>
												</Hint>
											{:else}
												<Hint placement="bottom" text="Type Answer">
													<button on:click={toggleShowExplanation} autofocus id="show-instruction-button" class="focus:outline-none focus:ring-2 ring-columbia p-1 rounded-sm mr-3 ">
														<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6 dark:invert">
															<path stroke-linecap="round" stroke-linejoin="round" d="M7.5 8.25h9m-9 3H12m-9.75 1.51c0 1.6 1.123 2.994 2.707 3.227 1.129.166 2.27.293 3.423.379.35.026.67.21.865.501L12 21l2.755-4.133a1.14 1.14 0 01.865-.501 48.172 48.172 0 003.423-.379c1.584-.233 2.707-1.626 2.707-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z" />
														</svg>
													</button>
												</Hint>
											{/if}
										</span>
									</div>
								</form>
						</div>
					{/each}
				</div>
			{:else}
				<h3 class="text-center font-mono font-bold text-columbia text-lg"> Well done! <br> You've completed today's quota, reviewing {stacks.done.length} cards. </h3>
			{/if}
		{/if}


		<!-- animated stacks -->
	{#if stacks && !showExplanation && ((windowHeight > 600 && !cardIsRevealed) || (windowHeight > 775 && cardIsRevealed)) && messages.length == 0}
		<div class="fixed left-1/2  -translate-x-1/2 h-1/4 px-2 w-2/3 lg:w-1/2 bottom-6 flex flex-row gap-16">
			<!-- whitespace -->
			<div class="border-b-2 border-columbia-dark dark:border-offwhite dark:opacity-50 w-full absolute bottom-0 left-1/2 -translate-x-1/2"></div>
				<!-- done -->
				<div class='basis-1/3 flex items-end {stacks.done.length > getStackHeight() ? "mb-1" : ""}'>
					<div class="flex flex-col-reverse w-full space-y-0">

						{#if stacks.done.length > getStackHeight()}
							<Hint placement="bottom" text="{stacks.done.length - getStackHeight()} advanced {stacks.done.length - getStackHeight() == 1 ? "card" : "cards"} hidden">
								<div class="w-full flex flex-col-reverse">
									
										<svg class="h-5 fill-offblack dark:fill-offwhite"
											fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
											<path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z"></path>
										</svg>
								</div>
							</Hint>
						

							{#each stacks.done.slice(-getStackHeight()) as id (id)}
								<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
									in:receive="{{key: id}}"
									out:send="{{key: id}}"
									animate:flip="{{duration: MOVE_DURATION}}"
								>
								</div>
							{/each}
						{:else}
							{#each stacks.done as id (id)}
									<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
										in:receive="{{key: id}}"
										out:send="{{key: id}}"
										animate:flip="{{duration: MOVE_DURATION}}"
									>
									</div>
							{/each}
						{/if}

					</div>
				</div>

				<!-- new -->
				<div class='basis-1/3 flex items-end {stacks.new.length > getStackHeight() ? "mb-1" : ""}'>
					<div class="flex flex-col-reverse w-full space-y-0">

						{#if stacks.new.length > getStackHeight()}
							<Hint placement="bottom" text="{stacks.new.length - getStackHeight()} new {stacks.new.length - getStackHeight() == 1 ? "card" : "cards"} hidden">
								<div class="w-full flex flex-col-reverse">
									
										<svg class="h-5 fill-offblack dark:fill-offwhite"
											fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
											<path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z"></path>
										</svg>
								</div>
							</Hint>
						

							{#each stacks.new.slice(-getStackHeight()) as id (id)}
								<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
									in:receive="{{key: id}}"
									out:send="{{key: id}}"
									animate:flip="{{duration: MOVE_DURATION}}"
								>
								</div>
							{/each}
						{:else}
							{#each stacks.new as id (id)}
									<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
										in:receive="{{key: id}}"
										out:send="{{key: id}}"
										animate:flip="{{duration: MOVE_DURATION}}"
									>
									</div>
							{/each}
						{/if}

					</div>
				</div>


				<!-- review -->
				<div class='basis-1/3 flex items-end {stacks.review.length > getStackHeight() ? "mb-1" : ""}'>
					<div class="flex flex-col-reverse w-full space-y-0">

						{#if stacks.review.length > getStackHeight()}
							<Hint placement="bottom" text="{stacks.review.length - getStackHeight()} review {stacks.review.length - getStackHeight() == 1 ? "card" : "cards"} hidden">
								<div class="w-full flex flex-col-reverse">
									
										<svg class="h-5 fill-offblack dark:fill-offwhite"
											fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
											<path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z"></path>
										</svg>
								</div>
							</Hint>
						

							{#each stacks.review.slice(-getStackHeight()) as id (id)}
								<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
									in:receive="{{key: id}}"
									out:send="{{key: id}}"
									animate:flip="{{duration: MOVE_DURATION}}"
								>
								</div>
							{/each}
						{:else}
							{#each stacks.review as id (id)}
									<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
										in:receive="{{key: id}}"
										out:send="{{key: id}}"
										animate:flip="{{duration: MOVE_DURATION}}"
									>
									</div>
							{/each}
						{/if}

					</div>
				</div>
			</div>
		{/if}
	</div>
	

		<!-- debugging stack height -->
			<!-- <div class="absolute right-5 bottom-0 dark:invert">
                {windowHeight}
			</div> -->
</div>
</div>

<style lang="postcss">
    .card-container {                                                           
        display: grid;                                                          
        grid-gap: 24px;                                                         
        justify-content: center;                                                
        padding: initial;                                                       
    }                       

	.card {                                                                     
        border: 0px solid #e1dfdd;                                              
        box-shadow: 0 10px 20px -8px rgba(197, 214, 214);                       
        transition: all 0.3s cubic-bezier(0, 0, 0.5, 1);                        
                                     
    }                          

	.left_navbar {
		height: calc(100vh + 200px);
	}



</style>
