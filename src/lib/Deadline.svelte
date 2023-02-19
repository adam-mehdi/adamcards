<script lang="ts">
	import Deck from './Deck.svelte';
	import {slide} from 'svelte/transition'
	import SettingsTray from './SettingsTray.svelte';
	import fsStore from '$lib/stores/fsStore'

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
	let settingsTrayOpen: boolean = false;

	{
		deadline_date
		deadline_time
	}


	function toggleExpanded() {
		expanded = !expanded;
		let entry = getEntry(path.split("~~"));
		entry.expanded = expanded
	}

	// find current entry in fsStore
	function getEntry(ancestors: string[]): FileSystemObject {
		let children: FileSystemObject[] = $fsStore;
		let entity: FileSystemObject = children[0];
		for (let name of ancestors) {
			const entries: FileSystemObject[] = children.filter((x: FileSystemObject) => x.name == name);

			if (entries.length == 0) {
				// create new folder at that path (automatically fires mkdir when mv)
				entries.push({
					entity_type: 'folder',
					name: name,
					files: [],
					expanded: true,
					deadline_date: null,
					deadline_time: null
				})
			} else if (entries.length > 1) {
				console.error("duplicate file system entity", name, path);
				break;
			} else {

				entity = entries.pop()!;
				children = entity.files!;
			}
		}
		return entity;
	}




</script>


<!-- folder container -->
<div class="relative"> 
	<div class="buttons flow-root">
		<span class:expanded class="pl-6 font-semibold dark:invert text-blacktext float-left {settingsTrayOpen ? "text-columbia dark:text-inverted-columbia font-extrabold" : ""}" on:click={toggleExpanded} on:keydown={toggleExpanded}>
			{name}
		</span>

		<SettingsTray entryType="deadline" path={path} bind:settingsTrayOpen />
		<!-- progress bar -->
		
		
	</div>



	


	{#if expanded && files !== null}
		<ul transition:slide={{duration:150}} class="border-l-[1px] border-separate rounded-lg ml-6 pl-2 border-columbia-dark dark:border-columbia">
					
			{#each files as file}
				<li class="first:pt-2">
					{#if file.entity_type === 'deck'}
						<Deck name={file.name} path={path.concat(`~~${file.name}`)} />

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
		background: url(/clock-open.png) 0 0.25em no-repeat;
		background-size: 1em 1em;
		cursor: pointer;
		background-size: 16px;
	}

	.expanded {
		background-image: url(/clock-closed.png);
		background-size: 16px;
	}




</style>

