<script lang="ts">
	import { page } from '$app/stores';
	// import KaTeXRenderer from '$lib/KaTeXRenderer.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	import { quintOut } from 'svelte/easing';                                      
    import { crossfade } from 'svelte/transition';                                 
    import { flip } from 'svelte/animate';                                         
                                                                                 
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
		studying: number | null
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
		// id2idx: Map<number, Card>,
		buf: CardBuffer,
	}



	// write buf_size - min_history every time
    const BUF_SIZE = 3;
	const MOVE_DURATION = 50;
	

	// initializes frontend and backend state
	let state: SessionState = {
		stacks: { 
			new: [],
			review: [],
			done: [],
			studying: null
		},
		card_is_revealed: false,
		session_is_finished: false,
		is_started: false,
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
			done: range(quotas.num_progressed).map(x => x + quotas.review_left),
			studying: null,
		}

		state = {
			stacks,
			card_is_revealed: false,
			session_is_finished: false,
			is_started: false,
			buf: {data: [], idx: -1}
		}

		// fill card buffer to begin review session
		drawCards();
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

	async function getNextCard() {
		// do not try to draw cards if session is done
		state.card_is_revealed = false;
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

		const study_card: undefined | number = stack.pop();
		if (study_card !== undefined)
			state.stacks.studying = study_card;
		else 
			console.error("UNDEFINED CARD IN STUDYING");

		// re-render the DOM
		state = state;
	}


	function handleResponse(response: number) {
		if (state.buf.idx < 0 || state.buf.data.length <= state.buf.idx) {
			console.log(state.buf, "producing a type error");
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

		const studying_id = state.stacks.studying!;
		let insert_idx = Math.floor(new_stack.length / 2);
		new_stack.splice(insert_idx, 0, studying_id);
		state.stacks.studying = null;

		// id2idx.set(buf_card.card.id, study_idx);

		// save and clear buffer if it is full
		if (state.buf.idx == state.buf.data.length - 1) {
			writeBuffer();
		}

		// get next card from the buffer
		getNextCard()
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
		let last_stack = get_stack_previous(curr_card);
		let studying = state.stacks.studying;
		last_stack.push(studying!);
		studying = null;

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
		state.stacks.studying = stack.pop()!;
		
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
		if (prev_card === undefined || prev_card.stack_after === undefined) {
			return;
		}

		let stack_after = get_stack_after(prev_card);
		stack_after.push(state.stacks.studying!);
		state.stacks.studying = null;

		// re-render the DOM
		state.stacks = state.stacks;
		
		state.buf.idx += 1;
		let curr_card = state.buf.data[state.buf.idx];

		let stack_before = get_stack_previous(curr_card);
		state.stacks.studying = stack_before.pop()!;

		// re-render the DOM
		state.stacks = state.stacks;

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
	function get_stack_previous(buf_card: ReviewSessionCard): number[] {
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
		invoke("save_card_buffer", 
			{"rcards": state.buf.data, "squotas": summedQuotas});
		state.buf = {data: [], idx: -1};
	}
	
	// called when no more cards or user exits
	async function cleanup() {
		await invoke('cleanup', { "cardBuffer": state.buf });
		state.session_is_finished = true;
	}


	function onKeyDown(e: KeyboardEvent) {

		// go back or forward through the buffer given user responses
		if (e.code == "ArrowLeft") { 
			getLastCard();
			state.stacks = state.stacks;
		} else if (e.code == "ArrowRight") {
			undoGetLastCard(); 
			state.stacks = state.stacks;
		}
		
		if (!state.is_started) {
			state.is_started = true;
			getNextCard();
		} else if (state.card_is_revealed) {
			// record a user response
			if (e.code == 'Digit1' || e.code == "KeyJ") 
				handleResponse(1);
			else if (e.code == 'Digit2' || e.code == "KeyK")
				handleResponse(2);
			else if (e.code == 'Digit3' || e.code == "KeyL")
				handleResponse(3);
		} else if (!state.card_is_revealed) {
			state.card_is_revealed = true;
		}
	} 
			



</script>


<!-- Home button -->
<a href="/"><button class="home-button" on:click={cleanup}>Home</button></a>


<!-- Listen for keyboard events -->
<svelte:window on:keydown|preventDefault={onKeyDown} />

<div class="whitespace space-y-32">

{#if !state.is_started}
<h3>Welcome, Cardwegian.</h3>


{:else} <!-- show card field if session has started, but not finished -->
	{#if !state.session_is_finished}


		<div class='board'>
			{#if state.stacks.studying !== null}
			<div class='top'>
				{#each [state.stacks.studying] as id (id)}
				
					<div class="card-container"
						in:receive="{{key: id}}"
						out:send="{{key: id}}"
						animate:flip="{{duration: 50}}">
						<div class="card">
							<!-- front of card -->
							<div class="front card-input">
								<textarea 
									class="card_field"
									bind:value={state.buf.data[state.buf.idx].card.fcard.front}></textarea>
							</div>

							<div class="card-hr" />

							<!-- back of card with buttons on toolbar -->
							<div class="back card-input {state.card_is_revealed ? '' : 'hidden'}">
								<textarea 
									bind:value={state.buf.data[state.buf.idx].card.fcard.back}></textarea>
								<br />
								<button class="hover:bg-sky-700" on:click={() => handleResponse(1)}>Hard</button>
								<button on:click={() => handleResponse(2)}>Okay</button>
								<button on:click={() => handleResponse(3)}>Good</button>
							</div>


						</div>

					</div>
				{/each}
			</div>
			{/if}
		</div>
	{:else}
	Congrats! You've completed your session today
	{/if}
{/if}




	<!-- Need to center contents -->
	<div class="flex space-x-4 ml-32">

	<!-- animated stacks -->
		<!-- done -->
		<div class='flex items-end align-middle'>
			<div>
			{#each state.stacks.done as id (id)}
				<div class="sm:w-20 md:w-28 lg:w-32 xl:w-40 rounded h-2 border-2 border-cyan-400"
					in:receive="{{key: id}}"
					out:send="{{key: id}}"
					animate:flip="{{duration: MOVE_DURATION}}"
				>
				
				</div>	
			{/each}
			<div class="bg-cyan-700 sm:w-20 md:w-28 lg:w-32 xl:w-40 rounded h-1"/>
			</div>
		</div>

		<!-- new -->
		<div class='flex items-end'>
			<div>

			{#each state.stacks.new as id (id)}
				<div class="sm:w-20 md:w-28 lg:w-32 xl:w-40 rounded h-2 border-2 border-rose-300"
					in:receive="{{key: id}}"
					out:send="{{key: id}}"
					animate:flip="{{duration: MOVE_DURATION}}"
				>
				
				</div>	
			{/each}
			<div class="bg-rose-700 sm:w-20 md:w-28 lg:w-32 xl:w-40 rounded h-1"/>
			</div>
		</div>

		<!-- review -->
		<div class='flex items-end'>
			<div>
			{#each state.stacks.review as id (id)}
				<div class="sm:w-20 md:w-28 lg:w-32 xl:w-40 rounded h-2 border-2 border-blue-700"
					in:receive="{{key: id}}"
					out:send="{{key: id}}"
					animate:flip="{{duration: MOVE_DURATION}}"
				>

				</div>	
			{/each}
			<div class="bg-blue-700 sm:w-20 md:w-28 lg:w-32 xl:w-40 rounded h-1"/>
			</div>
		</div>



	</div>

</div>


<style lang="postcss">

	/* Panel */
	.hidden {
		visibility: hidden;
	}


	.card_field {
		width: 312px; 
		height: 86px; 
		border-radius: 16px; 
		resize: None;
		font-size: 16px;
	}


	button {                                                                    
		cursor: pointer;
        border: none;                                                           
        height: 32px;                                                            
        border-radius: 0.3em;                                                   
		background-color: #e1dfdd;; 
		color: #1f1f1f;
    }    




	/* FROM EDIT */
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
        /* display: flex;                                                           */
        /* flex-direction: column;                                                  */
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

</style>
