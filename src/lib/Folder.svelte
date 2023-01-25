<script lang="ts">
	import Deadline from './Deadline.svelte'
	import {slide} from 'svelte/transition'
	import SettingsTrayButton from './SettingsTrayButton.svelte'



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

	// contains path to parents
	export let path: string = "";

	let settingsTrayOpen: boolean = false;
	function toggleSettingsTray() {
		settingsTrayOpen = !settingsTrayOpen;
	}


	function toggleExpanded() {
		expanded = !expanded;
	}


	

</script>


<!-- folder container -->
<div class=""> 
	<div class="flow-root h-6">
		<span class:expanded class="float-left" on:click={toggleExpanded} on:keydown={toggleExpanded}>
			{name}
		</span>
		<SettingsTrayButton entryType="folder" path={path} />
	</div>
	<hr class="ml-6 w-8"/>

{#if expanded && files !== null}
	<ul transition:slide={{duration:150}}>		
		{#each files as file}
			<li>
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
		font-weight: bold;
		cursor: pointer;
		background-size: 20px;
	}

	.expanded {
		background-image: url(/icons8-open-folder-32.png);
		background-size: 20px;
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

