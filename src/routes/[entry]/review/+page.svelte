<script lang="ts">
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';
	import { quintOut } from 'svelte/easing';                                      
    import { crossfade } from 'svelte/transition';                                 
    import { flip } from 'svelte/animate';                                         
	import { onDestroy } from 'svelte';
	import configStore from '$lib/stores/configStore'
	import Hint from 'svelte-hint';
	

	// write buf_size - min_history every time
	let isDarkMode = $configStore.is_dark_mode;
	let windowHeight = 0;
	$: getStackHeight = () => windowHeight > 650 ? Math.floor(windowHeight / 30) : Math.floor(windowHeight / 50);

	

    const BUF_SIZE = 5;
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

	// FrontendCard contains properties that can be edited in the frontend
	interface FrontendCard {
		id: number,
		front: string,
		back: string,
		deck_name: string,
	}

	// MetaData contains properties that are read-only fron the frontend
	interface MetaData {
		box_pos: number,
	}


	// Card tracks both frontend fields and data for backend algorithm and analysis
	interface Card {
		fcard: FrontendCard,
		md: MetaData
	}

	/**
	 * Data for animating cards
	 */

	// each number refers to one animated card on the screen
	// cards are moved on screen by moving them between arrays
	interface AnimatedCardStacks {
		new: number[],
		review: number[],
		done: number[],
		studying: number[]
	}


	/**
	 * Buffer for cards (serves as undo stack)
	 */ 

	interface CardBuffer {
		data: ReviewSessionCard[],
		idx: number,
	}

	interface ReviewSessionCard {
		card: Card,
		stack_before: "new" | "review",
		stack_after: "new" | "review" | "done" | null,
		user_response: number | null,
		box_pos_delta:  number | null,
	}

	// Quotas summed over all decks
	interface SummedQuotas {
		new_left: number,
		review_left: number,
		num_progressed: number,
	}

	/**
	 * State for review session
	 */
	interface SessionState {
		stacks: AnimatedCardStacks,
		card_is_revealed: boolean,      // back field is revealed
		session_is_finished: boolean,   // review session is completed
		is_started: boolean, 			// false if session has not started yet
		userAnswer: string,
		// id2idx: Map<number, Card>,
		buf: CardBuffer,
	}



	

	// initializes frontend and backend state
	let state: SessionState = {
		stacks: { 
			new: [],
			review: [],
			done: [],
			studying: []
		},
		card_is_revealed: false,
		session_is_finished: false,
		is_started: false,
		userAnswer: '',
		buf: { 
			data: [], 
			idx: -1 
		},
	};
	async function initState() {
		let entry = $page.params.entry;
		let quotas: SummedQuotas = await invoke('init_review_session', { "entryName": entry });

		let range = (n: number) => Array.from(Array(n).keys());
		let stacks: AnimatedCardStacks = {
			new: range(quotas.new_left),
			review: range(quotas.review_left).map(x => x + quotas.new_left),
			done: range(quotas.num_progressed).map(x => x + quotas.review_left + quotas.new_left),
			studying: [],
		}

		state = {
			stacks,
			card_is_revealed: false,
			session_is_finished: false,
			is_started: false,
			userAnswer: '',
			buf: {data: [], idx: -1}
		}

		// fill card buffer to begin review session
		await drawCards();
	}
	initState()

	
	async function drawCards() {
		
		
		// draw card from a backend deck
		const cards: Card[] = await invoke('draw_cards', { "numCards": BUF_SIZE });

		// put cards into card buffer
		let rcards: ReviewSessionCard[] = [];
		for (let card of cards) {
			rcards.push({
				card: card,
				stack_before: card.md.box_pos == 0 ? "new" : "review",
				stack_after: null,
				user_response: null,
				box_pos_delta: null
			});
		}

		// need to assign idxs to cards here TODO

		state.buf.data = rcards;
		state.buf.idx = -1;	
		
	}

	let card_drawn = false;
	async function getNextCard() {
		card_drawn = false;
		// do not try to draw cards if session is done
		state.card_is_revealed = false;
		state.userAnswer = '';
		if (state.session_is_finished)
			return;

		// draw cards if reached the end of the card state.buffer
		if (state.buf.idx == state.buf.data.length - 1) {
			await drawCards();
		}
		state.buf.idx += 1;

		// finish session if no more cards to review
		if (state.buf.data.length == 0) {
			cleanup();
			console.error("CLEANING UP");
			return;
		}

		let curr_card = state.buf.data[state.buf.idx];

		// remove card from proper stack and decrease quota
		let stack = curr_card.stack_before == "new" ? state.stacks.new 
			: state.stacks.review;

		
		if (stack.length <= 0)
			return;

		const study_card: undefined | number = stack.pop()!;
		state.stacks.studying.push(study_card);

		// re-render the DOM
		state.userAnswer = '';
		card_drawn = true;
		state.stacks = state.stacks;


	}


	let isFocusFire = true;
	async function handleResponse(response: number) {
		// prevent "Okay" from automatically firing		
		if (response == 2 && isFocusFire) {
			isFocusFire = false;
			return;
		}
		isFocusFire = true;


		if (state.buf.idx < 0 || state.buf.data.length <= state.buf.idx) {
			console.error(state.buf, "producing a type error");
		}

		let buf_card = state.buf.data[state.buf.idx];
		buf_card.user_response = response - 2;

		// compute box_pos_delta
		let box_pos_delta = response - 2;
		if (box_pos_delta == -1 && buf_card.card.md.box_pos <= 1)
			// no moving from review to new nor from new to negative
			box_pos_delta = 0;

		// move card box position
		buf_card.card.md.box_pos += box_pos_delta;
		buf_card.box_pos_delta = box_pos_delta;

		// find the stack where the card ends up
		if (box_pos_delta == 1) {
			buf_card.stack_after = "done";
		} else {
			buf_card.stack_after = buf_card.stack_before;
		}

		// put the card in that stack
		let new_stack = get_stack_after(buf_card);

		const studying_id = state.stacks.studying.pop()!;
		let insert_idx = Math.floor(new_stack.length / 2);
		new_stack.splice(insert_idx, 0, studying_id);

		// id2idx.set(buf_card.card.id, study_idx);

		// save and clear buffer if it is full
		if (state.buf.idx == state.buf.data.length - 1) {
			await writeBuffer();
		}

		// get next card from the buffer
		await getNextCard() // HAPPENDS IN HERE
		state.stacks = state.stacks;
	}

	// undo getNextCard, and reverse quotas state
	function getLastCard() {
		// return if buffer has not been initalized & session not started
		if (!state.is_started)
			return;

		// get current card being displayed
		let curr_card = state.buf.data[state.buf.idx];


		// no more cards to review if at the start of the buffer
		if (state.buf.idx == 0) {
			return;
		}
		
		// put card currently being studied back into the stack it was in
		let last_stack = get_stack_before(curr_card);
		let studying = state.stacks.studying.pop()!;
		last_stack.push(studying!);

		// re-render the DOM
		state.stacks = state.stacks;

		// no more cards to review if at the start of the buffer
		if (state.buf.idx == 0) {
			return;
		}
		// get previous card from the stack it ended up in and put it on display
		state.buf.idx -= 1 
		let prev_card = state.buf.data[state.buf.idx];
		// can be progressed

		let stack = get_stack_after(prev_card);
		state.stacks.studying.push(stack.pop()!);
		
		// re-render the DOM
		state.stacks = state.stacks;
	}

	// go forward in the buffer, assuming the already saved user responses
	function undoGetLastCard() {
		// return if buffer has not been initalized & session not started
		if (!state.is_started)
			return;
		

		let prev_card = state.buf.data[state.buf.idx];
		// can only undo if current card being studied has been reviewed
		if (prev_card === undefined || !prev_card.stack_after)
			return;
		

		let stack_after = get_stack_after(state.buf.data[state.buf.idx]);
		stack_after.push(state.stacks.studying.pop()!);

		// re-render the DOM
		state.stacks = state.stacks;
		
		state.buf.idx += 1;
		let curr_card = state.buf.data[state.buf.idx];

		let stack_before = get_stack_before(curr_card);
		state.stacks.studying.push(stack_before.pop()!);

		// re-render the DOM
		state.stacks = state.stacks;
		state.buf = state.buf;

	}

	// get stack to put the card into, where it will end up after a response
	function get_stack_after(buf_card: ReviewSessionCard): number[] {
		let new_stack;
		if (buf_card.stack_after == "new") { 
			new_stack = state.stacks.new; 
		} else if (buf_card.stack_after == "review") { 
			new_stack = state.stacks.review; 
		} else {
			new_stack = state.stacks.done;
		}
		return new_stack;
	}

	// get the stack to pop the card out of
	function get_stack_before(buf_card: ReviewSessionCard): number[] {
		return buf_card.stack_before == "new" 
			? state.stacks.new 
			: state.stacks.review;
	}

	async function writeBuffer() {
		let summedQuotas: SummedQuotas = {
			"new_left": state.stacks.new.length,
			"review_left": state.stacks.review.length,
			"num_progressed": state.stacks.done.length,
		}

		// TODO: remove summedQuotas; it is just used to do an assert
		await invoke("save_card_buffer", 
			{"rcards": state.buf.data, "squotas": summedQuotas});
		state.buf = {data: [], idx: -1};
	}
	
	// called when no more cards or user exits
	async function cleanup() {
		await invoke('cleanup', { "cardBuffer": state.buf });
		state.session_is_finished = true;
	}


	function onKeyDown(e: KeyboardEvent) {

		// detect whether user is editing card front or back (if they have focus)
		const activeElement = document.activeElement;
		const front = document.getElementById('front');
		const back = document.getElementById('back');
		// const bar = document.getElementById('user-answer-bar');
		// if user is editing card, don't trigger callbacks
		if (activeElement === front || activeElement === back) 
			return;
		
		// session starts or back is revealed if user presses enter
		if (e.code == "Enter") {
			if (!state.is_started) {
				state.is_started = true;
				getNextCard();
			} else {
				state.card_is_revealed = true;
			}
		}

		// allow typing in user answer bar
		// if (activeElement == bar)
		// 	return;
		
		// go back or forward through the buffer given user responses
		if (e.code == "<") 
			getLastCard();
		else if (e.code == ">")
			undoGetLastCard(); 
		
	} 
			
	function path2name(deck_path: string): string {
		let ancestors = deck_path.split("~~");
		// return ancestors[ancestors.length - 2] + "/" + ancestors[ancestors.length - 1];
		return ancestors[ancestors.length - 1];
	}

	function focus(el: any){
    	el.focus();
		state.userAnswer = '';
  	}

	function log(x: any): boolean {
		console.log(x);
		return false;

	}


	
	

</script>



<!-- Listen for keyboard events -->
<svelte:window on:keydown={onKeyDown} bind:innerHeight={windowHeight}/>
<!-- Home button -->
<div class={isDarkMode ? "dark" : ""}>
<div class="bg-offwhite dark:bg-offblack h-screen text-blacktext ">
	<div class="{windowHeight > 450 ? "h-5" : "h-2"}"></div>
	<div>

	<!-- bar at top -->
		<div class="ml-8 h-16 w-6 ">
			<a href="/" class="">
				<div on:click={cleanup} on:keypress={cleanup} class="fled justify-evenly w-6 ">
					<svg fill="highlight" class="flex-none h-6 w-6 cursor-pointer" 
						viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
						<path clip-rule="evenodd" fill-rule="evenodd" d="M9.293 2.293a1 1 0 011.414 0l7 7A1 1 0 0117 11h-1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-3a1 1 0 00-1-1H9a1 1 0 00-1 1v3a1 1 0 01-1 1H5a1 1 0 01-1-1v-6H3a1 1 0 01-.707-1.707l7-7z"></path>
					</svg>
				</div>
			</a>
		</div>


		<div class=" {windowHeight > 450 ? "space-y-32" : "space-y-8"}">

		{#if !state.is_started}
			<h3 class="text-center font-bold text-columbia text-4xl">Welcome, Cardwegian</h3>
			<h3 class="text-center font-mono font-bold text-columbia text-xl">Hit ENTER to begin</h3>


		{:else} <!-- show card field if session has started, but not finished -->
			{#if !state.session_is_finished}


				<div class=''>
					<!-- && state.stacks.studying !== null} -->
					{#if card_drawn} 
					<div class=''>
						{#each state.stacks.studying as id (id)}
						
							<div class="card-container"
								in:receive="{{key: id}}"
								out:send="{{key: id}}"
								animate:flip="{{duration: 50}}">
						            <!-- card fields -->
						            <div class="card flex flex-col h-full rounded-lg text-blacktext dark:text-whitetext">
				
						            	<div class="opacity-50 font-serif ml-4">
						            		{path2name(state.buf.data[state.buf.idx].card.fcard.deck_name)}
						            	</div>

						            		<!-- front field -->
						            		<div class="h-24 w-[520px] lg:w-[700px] m-4 text-inherit" >         
						            		<!-- md:w-[700px] lg:w-[800px] -->
						            			<textarea          
						            				class="h-24 w-[520px] lg:w-[700px] rounded-lg resize-none bg-offwhite  dark:bg-offblack text-inherit outline-columbia focus:outline outline-4 outline-offset-2 "                                                    
						            				bind:value={state.buf.data[state.buf.idx].card.fcard.front}                                    
						            			/>                                                              
						            		</div>          


						            	{#if state.card_is_revealed}
						            		<!-- rule separating front and back fields -->
						            		<div class="border-t border-1 border-columbia" />   
						            		<!-- back field -->
						            		<div class="h-24 w-[500px] lg:w-[700px] m-4 text-inherit" >         
						            			<!-- md:w-[700px] lg:w-[800px] -->
						            			<textarea          
						            				class="h-24 w-[520px] lg:w-[700px] rounded-lg resize-none bg-offwhite  dark:bg-offblack text-inherit outline-columbia focus:outline outline-4 outline-offset-2"                                                    
						            				bind:value={state.buf.data[state.buf.idx].card.fcard.back}                                    
						            			/>                                                              
						            		</div>          
						            			

											<!-- answer bar -->
						            		<div class="flex items-center justify-center top-[450px]">     
						            			<button 
						            				on:click={() => handleResponse(1)} 
						            				class="h-5 w-1/3 relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all duration-150 border-y border-l border-columbia rounded-bl-lg cursor-pointer group ease  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white">
						            				<span class="w-full h-0.5 absolute bottom-0 group-active:bg-transparent left-0 bg-gray-100"></span>
						            				<span class="h-full w-0.5 absolute bottom-0 group-active:bg-transparent right-0 bg-gray-100"></span>
						            				Hard 
						            			</button>

						            			<button
						            				on:click={() => handleResponse(2)}
						            				use:focus
						            				class="h-5 w-1/3 relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all duration-150 border-y border-l border-columbia cursor-pointer group ease  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white">
						            				<span class="w-full h-0.5 absolute bottom-0 group-active:bg-transparent left-0 bg-gray-100"></span>
						            				<span class="h-full w-0.5 absolute bottom-0 group-active:bg-transparent right-0 bg-gray-100"></span>
						            				Okay
						            			</button>      

						            			<button	
						            				class="h-5 w-1/3 relative z-30 inline-flex items-center justify-center px-8 py-3 overflow-hidden font-bold text-gray-500 transition-all duration-20 border-y border-r border-l border-columbia rounded-br-lg cursor-pointer group ease  outline-columbia focus:outline outline-4 outline-offset-2 bg-gradient-to-b from-offwhite dark:from-offblack to-gray-50 hover:from-gray-50 hover:to-white active:to-white"
						            				on:click={() => handleResponse(3)} >
						            				<span class="w-full h-0.5 absolute bottom-0 group-active:bg-transparent left-0 bg-gray-100"></span>
						            				<span class="h-full w-0.5 absolute bottom-0 group-active:bg-transparent right-0 bg-gray-100"></span>
						            				Good
						            			</button>    
						            		</div>       
						            	{/if}

						            </div>  

									<form class="w-full">
										<label class="w-full flex flex-wrap items-stretch relative mb-3">
											<span class="absolute left-1/4 inset-y-0 flex items-center pl-2">
												<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" 
													class="w-6 h-6 dark:invert">
													<path stroke-linecap="round" stroke-linejoin="round" d="M7.5 8.25h9m-9 3H12m-9.75 1.51c0 1.6 1.123 2.994 2.707 3.227 1.129.166 2.27.293 3.423.379.35.026.67.21.865.501L12 21l2.755-4.133a1.14 1.14 0 01.865-.501 48.172 48.172 0 003.423-.379c1.584-.233 2.707-1.626 2.707-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z" />
												  </svg>
												  
											</span>
											 
											<div class = "w-1/2 mx-auto">
												<input type="text"
												id="user-answer-bar"
												bind:value={state.userAnswer} 
												autofocus
												use:focus
												class="cursor-text rounded-lg h-8 placeholder:font-italic border w-full border-columbia py-2 pl-10 pr-4 focus:outline-none"
												placeholder="Your Answer"  />
					
											</div>
										</label>
									</form>


								</div>
						{/each}
					</div>
					{/if}
				</div>
			{:else}
			Congrats! You've completed your session today
			{/if}
		{/if}




			<div class="fixed left-1/2 -translate-x-1/2 h-1/3 px-2 w-2/3 lg:w-1/2 bottom-10 flex flex-row gap-16">
				<div class="border-b-2 border-columbia-dark dark:border-offwhite dark:opacity-50 w-full absolute bottom-0 left-1/2 -translate-x-1/2"></div>

			<!-- animated stacks -->
				<!-- done -->
				<div class='basis-1/3 flex items-end {state.stacks.done.length > getStackHeight() ? "mb-1" : ""}'>
					<div class="flex flex-col-reverse w-full space-y-0">

						{#if state.stacks.done.length > getStackHeight()}
							<Hint placement="bottom" text="{state.stacks.done.length - 23} new {state.stacks.done.length == 1 ? "card" : "cards"} hidden">
								<div class="w-full flex flex-col-reverse">
									
										<svg class="h-5 fill-offblack dark:fill-offwhite"
											fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
											<path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z"></path>
										</svg>
								</div>
							</Hint>
						

							{#each state.stacks.done.slice(-getStackHeight()) as id (id)}
								<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
									in:receive="{{key: id}}"
									out:send="{{key: id}}"
									animate:flip="{{duration: MOVE_DURATION}}"
								>
								</div>
							{/each}
						{:else}
							{#each state.stacks.done as id (id)}
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
				<div class='basis-1/3 flex items-end {state.stacks.new.length > getStackHeight() ? "mb-1" : ""}'>
					<div class="flex flex-col-reverse w-full space-y-0">

						{#if state.stacks.new.length > getStackHeight()}
							<Hint placement="bottom" text="{state.stacks.new.length - getStackHeight()} new {state.stacks.new.length == 1 ? "card" : "cards"} hidden">
								<div class="w-full flex flex-col-reverse">
									
										<svg class="h-5 fill-offblack dark:fill-offwhite"
											fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
											<path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z"></path>
										</svg>
								</div>
							</Hint>
						

							{#each state.stacks.new.slice(-getStackHeight()) as id (id)}
								<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
									in:receive="{{key: id}}"
									out:send="{{key: id}}"
									animate:flip="{{duration: MOVE_DURATION}}"
								>
								</div>
							{/each}
						{:else}
							{#each state.stacks.new as id (id)}
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
				<div class='basis-1/3 flex items-end {state.stacks.review.length > getStackHeight() ? "mb-1" : ""}'>
					<div class="flex flex-col-reverse w-full space-y-0">

						{#if state.stacks.review.length > getStackHeight()}
							<Hint placement="bottom" text="{state.stacks.review.length - 23} new {state.stacks.review.length == 1 ? "card" : "cards"} hidden">
								<div class="w-full flex flex-col-reverse">
									
										<svg class="h-5 fill-offblack dark:fill-offwhite"
											fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
											<path d="M10 3a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM10 8.5a1.5 1.5 0 110 3 1.5 1.5 0 010-3zM11.5 15.5a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z"></path>
										</svg>
								</div>
							</Hint>
						

							{#each state.stacks.review.slice(-getStackHeight()) as id (id)}
								<div class="border border-columbia-dark dark:border-black bg-columbia bg-opacity-57 h-2 rounded-lg text-xs"
									in:receive="{{key: id}}"
									out:send="{{key: id}}"
									animate:flip="{{duration: MOVE_DURATION}}"
								>
								</div>
							{/each}
						{:else}
							{#each state.stacks.review as id (id)}
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

			<div class="flex flex-row absolute bottom-0 left-4"	>
				


				<!-- backward chevron -->
				<div class="z-30 float-left">
					{#if state.buf.idx > 0}
						<Hint placement="top" text="Go forward">
							<button
								on:click={getLastCard}
								class="float-right cursor-pointer">
								<svg class="dark:invert flex-none h-7 w-7 cursor-pointer outline-columbia focus:outline outline-4 outline-offset-2 "
									fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
									<path clip-rule="evenodd" fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"></path>
								</svg>
							</button>
						</Hint>
					{:else if state.buf.idx < 1}
						<Hint placement="top" text="Reached end (shortcut `<`)">
							<div class="float-right cursor-default opacity-50">
								<svg class="dark:invert flex-none h-7 w-7 outline-columbia focus:outline outline-4 outline-offset-2 "
									fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
									<path clip-rule="evenodd" fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"></path>
								</svg>
							</div>
						</Hint>
					{/if}
				</div>

				<!-- forward chevron -->
				<div class="z-30 float-left">
					{#if state.buf.data[state.buf.idx] && state.buf.data[state.buf.idx].stack_after}
						<Hint placement="top" text="Go forward (shortcut `>`)>">
							<button
								on:click={undoGetLastCard}
								class="float-right cursor-pointer">
								<svg class="dark:invert flex-none h-7 w-7 cursor-pointer outline-columbia focus:outline outline-4 outline-offset-2 "
									fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
									<path clip-rule="evenodd" fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"></path>
								</svg>
							</button>
						</Hint>
					{:else}
						<Hint placement="top" text="On lastest card">
							<div
								class="float-right cursor-default opacity-50">
								<svg class="dark:invert flex-none h-7 w-7 outline-columbia focus:outline outline-4 outline-offset-2 "
									fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" data-darkreader-inline-fill="">
									<path clip-rule="evenodd" fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"></path>
								</svg>
							</div>
						</Hint>
					{/if}
				</div>

				
			</div>
			<div class="z-30 absolute right-2 bottom-1">
					<Hint placement="left" text="Stacks in order: Done | New | Review. Practice until all cards are in Done.">
						<div class="float-right cursor-default">
							<svg class="dark:invert flex-none h-6 w-6 outline-columbia focus:outline"
								fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
								<path clip-rule="evenodd" fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM8.94 6.94a.75.75 0 11-1.061-1.061 3 3 0 112.871 5.026v.345a.75.75 0 01-1.5 0v-.5c0-.72.57-1.172 1.081-1.287A1.5 1.5 0 108.94 6.94zM10 15a1 1 0 100-2 1 1 0 000 2z"></path>
							</svg>
						</div>
					</Hint>
				</div>

			<!-- <div class="absolute right-5 bottom-0 dark:invert">
                {windowHeight}
				({state.buf.idx} / {state.buf.data.length})
			</div> -->
				

		</div>


	</div>
</div>
</div>

<style lang="postcss">
    .card-container {                                                           
                                                                                 
        display: grid;                                                          
        grid-gap: 32px;                                                         
        justify-content: center;                                                
        padding: initial;                                                       
    }                       

	.card {                                                                     
                                                                                
        border: 0px solid #e1dfdd;                                              
        box-shadow: 0 10px 20px -8px rgba(197, 214, 214);                       
        transition: all 0.3s cubic-bezier(0, 0, 0.5, 1);                        
                                     
    }                                                                           
                                                                                


</style>
