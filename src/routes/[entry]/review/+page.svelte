<script lang="ts">
	import { page } from '$app/stores';
	// import KaTeX from '$lib/KaTeX.svelte';
	import KaTeXRenderer from '$lib/KaTeXRenderer.svelte';
	import { invoke } from '@tauri-apps/api/tauri';
	// import { crossfade } from 'svelte/tyes/runtime/transition';

	
	interface FrontendCard {
		id: number,
		front: string,
		back: string,
		deck_name: string
	}

	interface Quotas {
		new_left: number,
		review_left: number,
		num_progressed: number,
	}

	interface SessionState {
		card: FrontendCard | null,
		quotas: Quotas | null,
		is_revealed: boolean,    // back field is revealed
		is_finished: boolean,   // review session is completed
	}

	let state: SessionState = {
		card: null,
		quotas: null,
		is_revealed: false, 
		is_finished: false
	}

	async function initBoxes() {
		let entry = $page.params.entry;
		await invoke('init_boxes', { entry });

	}
	initBoxes();
	
	type DrawnItems = { card: FrontendCard, quotas: Quotas};
	async function drawCard() {
		// draw card from a backend deck
		const items: DrawnItems = await invoke('draw_card');
		state.card = items.card;
		state.quotas = items.quotas;

		// card.id == 0 ==> no more cards to review
		if (state.card && state.card.id == 0)
			cleanup();

		state.is_revealed = false;
	}
	drawCard();

	async function handleResponse(response: number) {
		await invoke("handle_response", {
			"card": state.card, 
			"quotas": state.quotas, 
			"response": response
		});
	}
	
	// called when no more cards or user exits
	async function cleanup() {
		await invoke('cleanup');
		state.is_finished = true;
	}

	function revealBack() {
		state.is_revealed = true;
	}

	const onKeyDown = (e: KeyboardEvent) => {
		// console.log(`Key Code: ${e.code}`);
		if (!state.is_revealed) {
			state.is_revealed = true;
		} else {
			if (e.code == 'Digit1') 
				handleResponse(1);
			else if (e.code == 'Digit2')
				handleResponse(2);
			else if (e.code == 'Digit3')
				handleResponse(3);
		}
	};
	console.log(state);


</script>

<!-- Listen for keyboard events -->
<svelte:window on:keydown|preventDefault={onKeyDown} />
<a href="/">Home</a>

{#if !state.is_finished}

	<div>
		<KaTeXRenderer input={state.card ? state.card.front.toString() : ''} />
	</div>
	<div class={state.is_revealed ? '' : 'hidden'}>
		<KaTeXRenderer input={state.card ? state.card.back.toString() : ''} />
	</div>

	<!-- <h3 class={back ? '' : 'hidden'}>{card ? card.back : ''}</h3> -->
	<br />
	<div class={state.is_revealed ? '' : 'hidden'}>
		<button on:click={() => handleResponse(1)}>Again</button>
		<button on:click={() => handleResponse(2)}>Hard</button>
		<button on:click={() => handleResponse(3)}>Good</button>
	</div>
{:else}
	<h1>Congrats! You've finished reviewing {$page.params.entry}</h1>
{/if}

<style>
	.hidden {
		visibility: hidden;
	}
</style>
