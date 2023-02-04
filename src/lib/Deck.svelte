<script lang="ts">
	import quotaStore from './stores/quotasStore';
	import SettingsTrayButton from './SettingsTray.svelte';

	export let name: string;
	export let path: string;
	let settingsTrayOpen = false;




</script>

<div class="flow-root">
	<span class="dark:invert text-blacktext float-left pl-6 {settingsTrayOpen ? "text-columbia dark:text-inverted-columbia font-extrabold" : ""}">
		{name}
	</span>

	<!-- SettingsTray -->
	<SettingsTrayButton entryType="deck" path={path} bind:settingsTrayOpen/>
	
</div>



<style>
	span {
		/* padding: 0 0 0 1.5em; */
		
		background: url(/icons8-king-of-spades-50.png) 0 .25em no-repeat;
		background-size: 16px;
	}
	
	
</style>