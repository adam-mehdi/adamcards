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
		stack_after?: "new" | "review" | "done" | null,
		box_pos_delta?:  number,
		user_response?: -1 | 0 | 1,
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
		// quotas: SummedQuotas,
		stacks: AnimatedCardStacks,
		card_is_revealed: boolean,      // back field is revealed
		session_is_finished: boolean,   // review session is completed
		is_started: boolean, 			// false if session has not started yet
		// id2idx
		buf: CardBuffer,
		// quotas_M_before: SummedQuotas
	}



	// write buf_size - min_history every time
	// const MIN_HISTORY;
    const BUF_SIZE = 2;
	const MOVE_DURATION = 50;
	

	// initializes frontend and backend state
	let state: SessionState;
	async function initState() {
		// let entry = $page.params.entry;
		// let quotas: SummedQuotas = await invoke('init_review_session', { "entryName": entry });

		const quotas: SummedQuotas = {
			new_left: 1,
			review_left: 1,
			num_progressed: 0
		};

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
		// const items: DrawnItems = await invoke('draw_cards', { BUF_SIZE });

		// HARD-CODE FOR TESTING
		const cards: Card[] = [
				// card 1
				{ 
					fcard: {
						id: 0,
						front: "front zero",
						back: "back zero",
						deck_name: "test~~adam",
					},
					md: {
						box_pos: 0,
					}
				},
				// card 2
				{
					fcard: {
						id: 1,
						front: "front one",
						back: "back one",
						deck_name: "test~~mehdi",
					},
					md: {
						box_pos: 1,
					},
				}
			]
		
		// finish session if no more cards to review
		if (cards.length == 0) {
			cleanup();
		}

		// card buffer
		let rcards: ReviewSessionCard[] = [];
		for (let card of cards) {
			rcards.push({
				card: card,
				stack_before: card.md.box_pos == 0 ? "new" : "review",
			});
		}

		state.buf = {
			data: rcards,
			idx: -1
		};
	}
	drawCards();


	async function getNextCard() {
		// draw cards if reached the end of the card state.buffer
		if (state.buf.idx == state.buf.data.length - 1) {
			drawCards()
		}
		state.buf.idx += 1;

		// finish session if no cards to draw from
		if (state.buf.data.length == 0)
			return
		
		
		let curr_card = state.buf.data[state.buf.idx];

		// remove card from proper stack and decrease quota
		let stack = curr_card.stack_before == "new" ? state.stacks.new 
			: state.stacks.review;
		state.stacks.studying = stack.pop()!;

		// re-render the DOM
		state = state;
	}


	function handleResponse(response: number) {
		let buf_card = state.buf.data[state.buf.idx];
		let box_pos = buf_card.card.md.box_pos;

		// compute box_pos_delta
		let box_pos_delta = response - 2;
		if (box_pos_delta == -1 && box_pos <= 1)
			// no moving from review to new nor from new to negative
			box_pos_delta = 0;

		// move card box position
		box_pos +=  box_pos_delta;
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
		if (prev_card.stack_after === undefined) {
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

	
	// called when no more cards or user exits
	async function cleanup() {
		// await invoke('cleanup');
		// state.session_is_finished = true;
	}

	function onKeyDown(e: KeyboardEvent) {
		// go back or forward through the buffer given user responses
		if (e.code == "ArrowLeft") { {
			getLastCard();
			state.stacks = state.stacks;
		}
			
		} else if (e.code == "ArrowRight") {
			undoGetLastCard(); 
			state.stacks = state.stacks;
		}
		
		if (!state.is_started) {
			state.is_started = true;
			getNextCard();

		}
			

		// record a user response
		if (!state.card_is_revealed)
			state.card_is_revealed = true;
		else {
			if (e.code == 'Digit1' || e.code == "KeyJ") 
				handleResponse(1);
			else if (e.code == 'Digit2' || e.code == "KeyK")
				handleResponse(2);
			else if (e.code == 'Digit3' || e.code == "KeyL")
				handleResponse(3);
		}
	};



</script>




<!-- Listen for keyboard events -->
<svelte:window on:keydown|preventDefault={onKeyDown} />


<!-- Home button -->
<a href="/">Home</a>
<!-- <a href="/"><button on:click={cleanup}>Home</button></a> -->

{#if !state.is_started}
<h3>Welcome. Press any key to begin.</h3>


{:else} <!-- show card field if session has started, but not finished -->
	{#if !state.session_is_finished}


<div class='board'>
	{#if state.stacks.studying !== null}
	<div class='top'>
		{#each [state.stacks.studying] as id (id)}
		
			<div class="card_display"
				in:receive="{{key: id}}"
				out:send="{{key: id}}"
				animate:flip="{{duration: 50}}">
				<!-- front of card -->
				<div>
					<textarea 
						class="card_field"
						bind:value={state.buf.data[state.buf.idx].card.fcard.front}></textarea>
					<!-- <KaTeXRenderer input={state.card ? state.card.front.toString() : ''} /> -->
				</div>

				<!-- back of card with buttons on toolbar -->
				<div class={state.card_is_revealed ? '' : 'hidden'}>
					<textarea 
						class="card_field"
						bind:value={state.buf.data[state.buf.idx].card.fcard.back}></textarea>
					<!-- <KaTeXRenderer input={state.card ? state.card.back.toString() : ''} /> -->
					<br />
					<button on:click={() => handleResponse(1)}>Hard</button>
					<button on:click={() => handleResponse(2)}>Okay</button>
					<button on:click={() => handleResponse(3)}>Good</button>
				</div>

			</div>
		{/each}
	</div>
	{/if}
</div>

	{/if}
{/if}


<div class="whitespace"></div>


<div class="stacks">

	<!-- animated stacks -->
		<!-- done -->
		<div class='done_stack'>
			{#each state.stacks.done as id (id)}
				<div class="card_in_stack"
					in:receive="{{key: id}}"
					out:send="{{key: id}}"
					animate:flip="{{duration: MOVE_DURATION}}"
				>
				
				</div>	
			{/each}
			<p>{state.stacks.done.length} done</p>
		</div>

		<!-- new -->
		<div class='new_stack'>
			{#each state.stacks.new as id (id)}
				<div class="card_in_stack"
					in:receive="{{key: id}}"
					out:send="{{key: id}}"
					animate:flip="{{duration: MOVE_DURATION}}"
				>
				
				</div>	
			{/each}
			<p>{state.stacks.new.length} new left </p>
		</div>

		<!-- review -->
		<div class='review_stack'>
			{#each state.stacks.review as id (id)}
				<div class="card_in_stack"
					in:receive="{{key: id}}"
					out:send="{{key: id}}"
					animate:flip="{{duration: MOVE_DURATION}}"
				>

				</div>	
			{/each}
			<p>{state.stacks.review.length} review left </p>
		</div>



</div>




<style>

	/* Panel */
	.hidden {
		visibility: hidden;
	}

	.card_display {
		margin: auto;
		width: 50%;
		padding: 10px;
	}

	.card_field {
		width: 312px; 
		height: 86px; 
		border-radius: 16px; 
		resize: None;
		font-size: 16px;
	}

	/* Whitespace */
	.whitespace {
		height: 120px;
	}

	/* Stacks */
	.stacks {
		max-width: 36em;
		margin: 0 auto;
	}

	.card_in_stack {
		width:80px;
		height:8px;
		border:1px solid grey;
		border-radius: 8px;
	}

	.done_stack {
		width: 40%;
		float: left;
		position: absolute;
		bottom: 100px;
		left: 200px;
	}
	.new_stack {
		width: 30%;
		float: left;
		padding: 0 1em 0 0;
		box-sizing: border-box;
		position: absolute;
		bottom: 100px;
		left: 400px;
	}

	.review_stack {
		width: 30%;
		float: left;
		padding: 0 1em 0 0;
		box-sizing: border-box;
		position: absolute;
		bottom: 100px;
		left: 600px;
	}

</style>
