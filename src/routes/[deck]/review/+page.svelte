<script lang="ts">
	import { page } from '$app/stores';

	let deckId = $page.params.deck;

	interface FrontBack {
		front: String;
		back: String;
	}

	let back: boolean = false;

	let card: FrontBack = {
		front: 'Hello',
		back: 'World'
	};

	// ask rust for the next card and tell them the deck we're studying, rust will make sure that
	// is the deck for which it is running a session?

	const onKeyDown = (e: KeyboardEvent) => {
		console.log(`Key Code: ${e.code}`);
		if (!back && (e.code == 'Space' || e.code == 'Enter')) {
			back = true;
		} else if (back) {
			if (e.code == 'Digit1') {
				console.log('Again');
			} else if (e.code == 'Digit2') {
				console.log('Good');
			} else if (e.code == 'Digit3') {
				console.log('Easy');
			}
		}
	};
</script>

<!-- Listen for keyboard events -->
<svelte:window on:keydown|preventDefault={onKeyDown} />
<p>Studying Deck: {deckId}</p>

<h3>{card.front}</h3>
<h3 class={back ? '' : 'hidden'}>{card.back}</h3>

<br />
<div>
	<button>Again</button>
	<button>Okay, KAWAIIII</button>
	<button>SOO TRUEEEE</button>
</div>
<a href="/">Home</a>

<style>
	.hidden {
		visibility: hidden;
	}
</style>
