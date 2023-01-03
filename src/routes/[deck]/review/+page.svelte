<script lang="ts">
	import { page } from '$app/stores';
	import { invoke } from '@tauri-apps/api/tauri';

	interface FrontendCard {
		id: number;
		front: String;
		back: String;
	}

	function isFrontendCard(object: any): boolean {
		return true;
	}

	interface GetCardError {
		type: 'NoCardsToReview' | 'OtherError';
		message: string;
	}

	type GetCardResponse =
		| { type: 'FrontendCard'; card: FrontendCard }
		| { type: 'NoCardsToReview' }
		| { type: 'GetCardError'; message: string };

	// ask the backend to give me the next card to review from a given deck
	const getCard = async (deckId: number): Promise<GetCardResponse> => {
		try {
			// Try to get the next card
			let c: FrontendCard = await invoke('get_next_card', { deckId });
			return { type: 'FrontendCard', card: c };
		} catch (err) {
			if (typeof err === 'string') {
				if (err === 'NoCardsToReview') {
					return { type: 'NoCardsToReview' };
				} else {
					return { type: 'GetCardError', message: err };
				}
			} else {
				return {
					type: 'GetCardError',
					message: 'unexpected non-string value returned from get_next_card'
				};
			}
		}
	};

	const postReview = (reviewScore: number) => {
		invoke('post_review', { deckId: deckId, reviewScore: reviewScore });
	};

	const handleReviewResponse = (reviewScore: number) => {
		// synchronously
		// post the review
		// get the next card
		postReview(reviewScore);
		back = false;
		handleGetCard();
	};

	// SETUP
	let deckId = Number($page.params.deck);

	let back: boolean = false;
	let done: boolean = false;

	let card: FrontendCard | null;
	card = null;
	const handleGetCard = () => {
		getCard(deckId).then((result) => {
			if (result.type === 'FrontendCard') {
				card = result.card;
			} else if (result.type === 'NoCardsToReview') {
				console.log('No Cards to review');
				done = true;
			} else if (result.type === 'GetCardError') {
				console.error(result.message);
			} else {
				// error
			}
		});
	};

	handleGetCard();

	// We should have some check for if there are no cards left to review that is faster
	// than initializing a whole review session...

	// ask rust for the next card and tell them the deck we're studying, rust will make sure that
	// is the deck for which it is running a session?

	const onKeyDown = (e: KeyboardEvent) => {
		console.log(`Key Code: ${e.code}`);
		if (!back && (e.code == 'Space' || e.code == 'Enter')) {
			back = true;
		} else if (back) {
			if (e.code == 'Digit1') {
				console.log('Again');
				handleReviewResponse(1);
			} else if (e.code == 'Digit2') {
				console.log('Good');
				handleReviewResponse(2);
			} else if (e.code == 'Digit3') {
				console.log('Easy');
				handleReviewResponse(3);
			}
		}
	};
</script>

<!-- Listen for keyboard events -->
<svelte:window on:keydown|preventDefault={onKeyDown} />
<a href="/">Home</a>
<p>Studying Deck: {deckId}</p>

{#if !done}
	<h3>{card ? card.front : ''}</h3>
	<h3 class={back ? '' : 'hidden'}>{card ? card.back : ''}</h3>
	<br />
	<div class={back ? '' : 'hidden'}>
		<button on:click={() => handleReviewResponse(1)}>Again</button>
		<button on:click={() => handleReviewResponse(2)}>Good</button>
		<button on:click={() => handleReviewResponse(3)}>Easy</button>
	</div>
{:else}
	<h1>No more cards to review for deck {deckId}!</h1>
{/if}

<style>
	.hidden {
		visibility: hidden;
	}
</style>
