<script lang="ts">
	import Deck from './Deck.svelte';
	import {slide} from 'svelte/transition'
	import SettingsTrayButton from './SettingsTrayButton.svelte';

	type FileSystemObject = {
		entity_type: 'folder' | 'deadline' | 'deck';
		name: string;
		files: FileSystemObject[] | null;
		expanded: boolean | null;
		deadline_date: string | null;
		deadline_time: string | null;
	};
	
	export let name: string = '';
	export let files: null | FileSystemObject[];
	export let expanded: boolean | null;
	export let deadline_date: string | null; // YYYY-M-DD format
	export let deadline_time: string | null; // HH:HH format (military time)
	export let path: string;

	{
		deadline_date
		deadline_time
	}

	function toggleExpanded() {
		expanded = !expanded;
	}

</script>


<!-- folder container -->
<div class="folder-container"> 
	<div class="buttons flow-root">
		<span class:expanded class="float-left" on:click={toggleExpanded} on:keydown={toggleExpanded}>
			{name}
		</span>
		<SettingsTrayButton entryType="deadline" path={path} />
	</div>

{#if expanded && files !== null}
	<ul transition:slide={{duration:100}}>
				
		{#each files as file}
			<li>
				{#if file.entity_type === 'deck'}
					<Deck name={file.name} path={path.concat(`~~${file.name}`)}/>

				{:else}
					{console.error("Deadlines must hold decks and only decks")}
				{/if}
			</li>
		{/each}
	</ul>
{/if}
</div>


<style lang="postcss">
	span {
		padding: 0 0 0 1.5em;
		background: url(/icons8-closed-book-78.png) 0 0.1em no-repeat;
		background-size: 1em 1em;
		font-weight: bold;
		cursor: pointer;
		background-size: 24px;
	}

	.expanded {
		background-image: url(/icons8-open-book-30.png);
		background-size: 16px;
	}

	ul {
		padding: 0.2em 0 0 0.5em;
		margin: 0 0 0 0.5em;
		list-style: none;
		border-left: 1px solid #eee;
	}

	li {
		padding: 0.2em 0;
	}
	.folder-container {
		position: relative;
	}


	ul {
		padding: 0.2em 0 0 0.5em;
		margin: 0 0 0 0.5em;
		list-style: none;
		border-left: 1px solid #eee;
	}
	li {
		padding: 0.2em 0;
	}
</style>

