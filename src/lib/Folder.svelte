<script lang="ts">
	import Deadline from './Deadline.svelte'
	import {slide} from 'svelte/transition'
	import SettingsTrayButton from './SettingsTray.svelte'
	import fsStore from '$lib/stores/fsStore'



	// import { files } from '$service-worker';
	let slideDuration = 150;

	type FileSystemObject = {
		entity_type: 'folder' | 'deadline' | 'deck';
		name: string;
		files: FileSystemObject[] | null;
		expanded: boolean | null;
		deadline_date: string | null;
		deadline_time: string | null;
	};
	

	export let name: string = '';
	export let files: FileSystemObject[] | null = [];
	export let expanded: boolean | null = false;
	let settingsTrayOpen: boolean = false;

	// contains path to parents
	export let path: string = "";


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
<div class="w-full"> 
	<div class="flow-root h-6">
		<span class:expanded class="font-bold dark:invert text-blacktext float-left {settingsTrayOpen ? "text-columbia dark:text-inverted-columbia font-extrabold" : ""}" on:click={toggleExpanded} on:keydown={toggleExpanded}>
			{name}
		</span>
		<SettingsTrayButton entryType="folder" path={path} bind:settingsTrayOpen />
	</div>
	<hr class=" w-10 text-columbia"/>

{#if expanded && files !== null}
	<ul transition:slide={{duration:slideDuration}} class="">		
		{#each files as file}
			<li class="max-w-xl">
				{#if file.entity_type == 'folder'}
					<svelte:self 
						name={file.name} 
						files={file.files} 
						expanded={file.expanded}
						path={path.concat(`~~${file.name}`)}
					/>


				{:else if file.entity_type == 'deadline'}
					<Deadline 
						name={file.name} 
						files={file.files} 
						deadline_date={file.deadline_date}
						deadline_time={file.deadline_time}
						expanded={file.expanded}
						path={path.concat(`~~${file.name}`)}
					/>

				{:else}
					{console.error("folders must contain either folders or deadlines, not decks")}
				{/if}
			</li>
		{/each}
	</ul>
{/if}

</div>

<style lang="postcss">
	span {
		padding: 0 0 0 1.75em;
		background: url(/icons8-folder-24.png) 0 0.1em no-repeat;
		cursor: pointer;
		background-size: 18px;
	}

	.expanded {
		background-image: url(/icons8-open-folder-32.png);
		background-size: 20px;
	}


	ul {
		padding: 0.3em 0 0 0.8em;
		margin: 0 0 0 .8em;
		/* border-left: 1px solid; */
	}
	li {
		padding: 0.2em 0;
	}
</style>

