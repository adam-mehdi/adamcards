<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';

    // import { writable } from "svelte/store";
	import Hint from 'svelte-hint';
  
    let apiKey = "";
    export let apiKeyPresent: boolean;
    export let submittedApiKey = false;
    let invalidApiKeySubmitted = false

    
    async function saveApiKey() {

        // first check if apiKey is valid
        if (await testOpenAiApiKey(apiKey)) {

            // now save apiKey
            invoke('write_api_key', { apiKey });        
            apiKeyPresent = true;
            submittedApiKey = true;

        } else {
            // apiKey not valid
            invalidApiKeySubmitted = true;
            
        }


    }


    let loadingApiKey = false
    async function testOpenAiApiKey(apiKey: string): Promise<boolean> {
        const apiUrl = 'https://api.openai.com/v1/chat/completions';

        const chatRequestOpts = {
            model: 'gpt-3.5-turbo',
            messages: [
                {
                    role: 'system',
                    content: 'You are a helpful assistant.',
                },
                {
                    role: 'user',
                    content: 'Who won the world series in 2020?',
                },
            ],
            max_tokens: 5,
        };

        const requestOptions: RequestInit = {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${apiKey}`,
            },
            body: JSON.stringify(chatRequestOpts),
        };

        try {
            loadingApiKey = true
            const response = await fetch(apiUrl, requestOptions);
            loadingApiKey = false

            if (response.ok) {
                console.log('API key is valid');
                return true;
            } else {
                console.log('API key is invalid or there was an issue with the request');
                // const errorDetails = await response.json();
                // console.log('Error details:', errorDetails);
                return false;
            }
        } catch (error) {
            console.error('Error:', error);
            return false;
        }
    }

  </script>
  
  <div class="w-full py-2 px-6 mt-5 bg-offwhite dark:bg-offblack dark:text-whitetext border p-4 rounded-lg">
    <div class="flex flex-row mb-1 font-mono text-sm" >
        <Hint placement="top" text="See adamcards.com/#/guides for help">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-6 h-5 mr-2 cursor-help">
                <path fill-rule="evenodd" d="M9 4.5a.75.75 0 01.721.544l.813 2.846a3.75 3.75 0 002.576 2.576l2.846.813a.75.75 0 010 1.442l-2.846.813a3.75 3.75 0 00-2.576 2.576l-.813 2.846a.75.75 0 01-1.442 0l-.813-2.846a3.75 3.75 0 00-2.576-2.576l-2.846-.813a.75.75 0 010-1.442l2.846-.813A3.75 3.75 0 007.466 7.89l.813-2.846A.75.75 0 019 4.5zM18 1.5a.75.75 0 01.728.568l.258 1.036c.236.94.97 1.674 1.91 1.91l1.036.258a.75.75 0 010 1.456l-1.036.258c-.94.236-1.674.97-1.91 1.91l-.258 1.036a.75.75 0 01-1.456 0l-.258-1.036a2.625 2.625 0 00-1.91-1.91l-1.036-.258a.75.75 0 010-1.456l1.036-.258a2.625 2.625 0 001.91-1.91l.258-1.036A.75.75 0 0118 1.5zM16.5 15a.75.75 0 01.712.513l.394 1.183c.15.447.5.799.948.948l1.183.395a.75.75 0 010 1.422l-1.183.395c-.447.15-.799.5-.948.948l-.395 1.183a.75.75 0 01-1.422 0l-.395-1.183a1.5 1.5 0 00-.948-.948l-1.183-.395a.75.75 0 010-1.422l1.183-.395c.447-.15.799-.5.948-.948l.395-1.183A.75.75 0 0116.5 15z" clip-rule="evenodd" />
            </svg>
        </Hint>
        

        {#if loadingApiKey}
            <h2 class="mb-2 font-mono text-sm">  Verifying your API Key... Hang tight! 🔐</h2>
        {:else if !apiKeyPresent && !invalidApiKeySubmitted}
            <h2 class="mb-2 font-mono text-sm">Enter your OpenAI API Key</h2>
        {:else if invalidApiKeySubmitted}
            <h2 class="mb-2 font-mono text-sm">Invalid API Key. Resubmit new key</h2>
        {:else}
            <h2 class="mb-2 font-mono text-sm">API Key works! You're all ready to go</h2>
        {/if}

    </div>
    <input
      type="text"
      bind:value="{apiKey}"
      placeholder="Secret API Key"
      class="h-8 w-full mb-2 hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-3 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring-2 duration-75"
    />
    <button
      on:click="{saveApiKey}"
      class="h-8 text-sm mb-1 w-full hover:bg-columbia dark:hover:bg-columbia-dark dark:bg-offblack border-2 border-columbia rounded-lg block px-4 dark:hover:text-whitetext ring-columbia focus:outline-none focus:ring-2 duration-75"
    >
      {apiKeyPresent ? "Reset" : "Save"} API Key
    </button>
  </div>
  
  