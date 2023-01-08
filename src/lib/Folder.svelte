<script lang="ts">
	import File from './File.svelte';
	// import KaTeXRenderer from './KaTeXRenderer.svelte';
	import settingsGear from '$lib/images/settings-gear-black.png';
	import { clickOutside } from './actions/click_outside';

	type FileSystemObject = {
		type: 'folder' | 'file';
		name: string;
		files?: FileSystemObject[];
		expanded?: boolean;
		deadlineString?: string | null;
	};

	export let expanded = false;
	export let name: string = '';
	export let type: 'folder' | 'file' = 'folder';
	export let deadlineString: string | null = null;
	export let files: FileSystemObject[];
	let foldersMuted: boolean = false;

	function toggle() {
		if (!foldersMuted) {
			expanded = !expanded;
		}
	}

	let settingsTrayOpen: boolean = false;
	function toggleSettingsTray() {
		settingsTrayOpen = !settingsTrayOpen;
	}

	let deadlineEditable: boolean = false;

	function handleDeadlineClick() {
		deadlineEditable = true;
		foldersMuted = true;
	}

	// Write out click logic for the deadline string
	function handleOutclick() {
		deadlineEditable = false;
		foldersMuted = false;
	}

	function focusOnMount(el) {
		el.focus();
	}

	function handleEditableDeadlineKeypress(e: KeyboardEvent) {
		if (e.code === 'Enter') {
			deadlineEditable = false;
		}
	}

	// Deadline Should Also Save on ENTER
</script>

<div class="folder-container">
	<span class="folder" class:expanded on:click={toggle} on:keypress={toggle}>
		<span class="span-left">
			{name}
		</span>
		<span class="span-right">
			{#if deadlineEditable}
				<span
					on:click|stopPropagation
					on:keypress|stopPropagation={handleEditableDeadlineKeypress}
					use:clickOutside={deadlineEditable}
					on:outclick={handleOutclick}
				>
					<input type="text" bind:value={deadlineString} use:focusOnMount />
				</span>
			{:else}
				<!-- UPDATE THIS  -->
				<span
					on:keydown|stopPropagation={() => {
						!deadlineEditable && handleDeadlineClick();
					}}
					on:click|stopPropagation={() => {
						!deadlineEditable && handleDeadlineClick();
					}}>{deadlineString ? deadlineString : 'No Deadline Set'}</span
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
				<li class="settings-tray-option">Edit Deadline</li>
				<li class="settings-tray-option">Edit Deck Name</li>
				<li class="settings-tray-option">Edit Deck</li>
				<li class="settings-tray-option">Delete Deck</li>
			</ul>
		</div>
	{/if}
</div>

{#if expanded}
	<ul>
		{#each files as file, i}
			<li>
				{#if file.type === 'folder'}
					<svelte:self
						bind:files={file.files}
						bind:name={file.name}
						bind:expanded={file.expanded}
						bind:deadlineString={file.deadlineString}
						type="folder"
					/>
				{:else}
					<File {...file} />
				{/if}
			</li>
		{/each}
		<li>
			{#if type === 'folder'}
				<span class="create-file-folder-row">
					<span class="span-left"><button>+ File</button></span>
					<span class="span-right" />
				</span>
				<span class="create-file-folder-row">
					<span class="span-left"><button>+ Folder</button></span>
					<span class="span-right" />
				</span>
			{/if}
		</li>
	</ul>
{/if}

<style>
	.folder-container {
		position: relative;
	}

	.folder {
		display: flex;
		justify-content: space-between;
		align-items: center;
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

	.create-file-folder-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
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

	.create-file-folder-row:hover {
		background-color: rgb(245, 245, 245);
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
