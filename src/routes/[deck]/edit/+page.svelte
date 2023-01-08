<script lang="ts">
	import { flip } from 'svelte/animate'
	import  Search  from '$lib/SearchBarFilter.svelte'

	const dragDuration = 300
	let cards: number[] = Array(20).fill(1).map((_, i) => i + 1)
	let draggingCard: number | undefined;
	let animatingCards = new Set()

	function swapWith(card: number) {
		if (draggingCard === card || animatingCards.has(card)) return
		animatingCards.add(card)
		setTimeout(() => animatingCards.delete(card), dragDuration)
		const cardAIndex = cards.indexOf(draggingCard!)
		const cardBIndex = cards.indexOf(card)
		cards[cardAIndex] = card
		cards[cardBIndex] = draggingCard!
	}
	let front = 'adam';
	let back = '';
	
	let prompt = '';
	
	function createCard(event: any) {
		// append to cards
		// crossfade animation
		// clear field
        
	}
	
	const filterCards = () => {	
		// return array of cards that match search term
	}
	
</script>
<a href="/">Home</a>

<div class="center_panel">
	<div class="center_card">
			<div class="front">
				<textarea class="center_text" bind:value={front} />
			</div>
			
		<div class="back">
				<textarea class="center_text" bind:value={back} />
		</div>
		
		
		
		<div class="center_bar">
		<!-- 	if editing; specify deadline if creating -->
			<Search bind:prompt on:input={filterCards} />
			<button on:click={createCard}> >> </button>
			<button on:click={createCard}> |--> </button>
		</div>
			
	</div>
</div>

<div class="container">
	{#each cards as card (card)}
		<div
			animate:flip={{ duration: dragDuration }}
			class="card"
			draggable="true"
			on:dragstart={() => draggingCard = card}
			on:dragend={() => draggingCard = undefined}
		  on:dragenter={() => swapWith(card)}
			on:dragover|preventDefault
		>
			<div class="front">
				<textarea bind:value={front} />
			</div>
			
		<div class="back">
				<textarea bind:value={back} />
		</div>
			
		</div>
	{/each}
</div>

<style>
	.container {
		display: grid;
/* format depends on size of window		 */
		grid-template-rows: repeat(4, 1fr);
		grid-template-columns: repeat(2, 1fr);
		gap: 8px;
	}

	.card {
		display: flex;
 		flex-direction:column;
		padding: 8px;
		justify-content: center;
		align-items: center;
    color: darkblue;
		
		width: 100%;
		height: 100%;
		font-size: 1.5rem;
	}
	
	.front {
		border-radius: 16px; 
		/* background-color: ivory; */
	}
	
	.back {
		border-radius: 16px; 
		/* background-color: #B9D9EB; */
	}
	
	textarea { 
		width: 256px; 
		height: 64px; 
		border-radius: 16px; 
		resize: None;
		font-size: 12px;
		font-family:"Times";
	}
	

	/* add image support and submit bar at bottom	 */
	.center_panel {
		height: 256px;
	}
	
	.center_card {
		display: flex;
 		flex-direction:column;
		padding: 8px;
		justify-content: center;
		align-items: center;
		
		width: 100%;
		height: 100%;
		font-size: 1.5rem;
		
	}
	
	.center_text {
		width: 312px; 
		height: 86px; 
		border-radius: 16px; 
		resize: None;
		font-size: 16px;
	}
	
	.center_bar {
		display: flex;
		/* background-color:ivory; */
		height: 32px;
	}
	
	button {
		height: 32px;
		font-size: 12px;
	}
	
</style>