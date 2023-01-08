<script lang="ts">
	import File from './File.svelte';
	import KaTeXRenderer from './KaTeXRenderer.svelte';
	import settingsGear from '$lib/images/settings-gear-black.png';

	export let expanded = false;
	export let name: string = '';

	type FileSystemObject = {
		type: 'folder' | 'file';
		name: string;
		files?: FileSystemObject[];
		expanded?: boolean;
	};

	export let files: FileSystemObject[];

	function toggle() {
		expanded = !expanded;
	}

	let settingsTrayOpen: boolean = false;
	function toggleSettingsTray() {
		settingsTrayOpen = !settingsTrayOpen;
	}

	let deadline_editable: boolean = false;
	function handleDeadlineClick() {
		deadline_editable = true;
	}

	// Write out click logic for the deadline string
</script>

<div class="folder-container">
	<span class="folder" class:expanded on:click={toggle} on:keypress={toggle}>
		<span class="span-left">{name}</span>
		<span class="span-right">
			{#if deadline_editable}
				<input type="text" value="edit me" on:click|stopPropagation />
			{:else}
				<span
					on:keydown|stopPropagation={handleDeadlineClick}
					on:click|stopPropagation={handleDeadlineClick}>Deadline String</span
				>
			{/if}
			<span>
				<button
					class="folder-settings-button"
					on:keydown|stopPropagation={toggleSettingsTray}
					on:click|stopPropagation={toggleSettingsTray}
				>
					<img class="gear" src={settingsGear} alt="setting gear" />
				</button></span
			>
		</span>
	</span>
	{#if settingsTrayOpen}
		<div class="settings-tray">
			<ul>
				<li><button>fdjfks</button></li>
				<li><button>Thing</button></li>
				<li><button>Thing</button></li>
				<li><button>Thing</button></li>
			</ul>
		</div>
	{/if}
</div>

{#if expanded}
	<ul>
		{#each files as file, i}
			<li>
				{#if file.type === 'folder'}
					<svelte:self {...file} bind:expanded={file.expanded} />
				{:else}
					<File {...file} />
				{/if}
			</li>
		{/each}
	</ul>
{/if}

<style>
	.folder-container {
		position: relative;
	}

	.folder {
		display: flex;
		justify-content: space-between;
		width: 100%;
		/* padding: 2px 2px 2px 1.5em; */
		/* background: url(/folder.svg) 0 0.1em no-repeat; */
		padding: 2px 10px;
		background: 0 0.1em no-repeat;
		background-size: 1em 1em;
		font-weight: bold;
		cursor: pointer;
		margin: 1px;
		border-radius: 2px;
	}

	.folder:hover {
		background-color: #eee;
	}

	.span-right {
		display: flex;
		justify-content: space-between;
		width: 250px;
	}
	.span-left {
		width: 33%;
	}

	/* .expanded {
		background-image: url(/folder-open.svg);
	} */

	ul {
		padding: 0.2em 0 0 0.5em;
		margin: 0 0 0 0.5em;
		list-style: none;
		border-left: 1px solid #eee;
	}

	li {
		padding: 0.2em 0;
	}

	.gear {
		height: 1em;
		width: 1em;
	}

	.gear:hover {
		height: 1.2em;
		width: 1.2em;
	}

	.folder-settings-button {
		height: 100%;
		width: 2em;
		border: none;
		display: flex;
		justify-content: center;
		align-items: center;
		background-color: transparent;
		border-radius: 100px;
	}

	.settings-tray {
		padding: 10px;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		position: absolute;
		right: 0;
		top: 2em;
		width: 150px;
		background-color: #eee;
		box-shadow: 0px 0px 10px #eee;
		border-radius: 2px;
		z-index: 4;
	}

	.settings-tray ul {
		padding: 0;
		margin: 0;
		font-size: 0.8em;
	}

	.settings-tray li {
		border: none;
		padding: none;
	}
</style>
