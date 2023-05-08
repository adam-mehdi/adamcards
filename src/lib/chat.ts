import type { CreateChatCompletionRequest, ChatCompletionRequestMessage } from 'openai'
import type { RequestHandler } from '../routes/api/chat/$types'
import { getTokens } from '$lib/tokenizer'
import { json } from '@sveltejs/kit'

export async function processChatRequest(requestData: any) {	
	try {

		if (!requestData) {
			throw new Error('No request data');
		  }


		const reqMessages: ChatCompletionRequestMessage[] = requestData.messages
		const OPENAI_API_KEY = requestData.apiKey

		if (!reqMessages) {
			throw new Error('no messages provided')
		}

		let tokenCount = 0

		reqMessages.forEach((msg) => {
			const tokens = getTokens(msg.content)
			tokenCount += tokens
		})

		const moderationRes = await fetch('https://api.openai.com/v1/moderations', {
			headers: {
				'Content-Type': 'application/json',
				'Authorization': `Bearer ${OPENAI_API_KEY}`
			},
			method: 'POST',
			body: JSON.stringify({
				input: reqMessages[reqMessages.length - 1].content
			})
		})
		if (!moderationRes.ok) {
			const err = await moderationRes.json()
			throw new Error(err.error.message)
		}

		const moderationData = await moderationRes.json()
		const [results] = moderationData.results

		if (results.flagged) {
			throw new Error('Query flagged by openai')
		}

		const prompt = requestData.systemPrompt
		tokenCount += getTokens(prompt)

		if (tokenCount >= 1000) {
			throw new Error('Query too large')
		}

		const messages: ChatCompletionRequestMessage[] = [
			{ role: 'system', content: prompt },
			...reqMessages
		]

		const max_tokens = requestData.maxTokens ? requestData.maxTokens : 150
		const chatRequestOpts: CreateChatCompletionRequest = {
			model: 'gpt-3.5-turbo',
			messages,
			temperature: 0.5,
			stream: true,
            max_tokens,
			frequency_penalty: 1.0
		}

		const chatResponse = await fetch('https://api.openai.com/v1/chat/completions', {
			headers: {
				Authorization: `Bearer ${OPENAI_API_KEY}`,
				'Content-Type': 'application/json'
			},
			method: 'POST',
			body: JSON.stringify(chatRequestOpts)
		})

		if (!chatResponse.ok) {
			const err = await chatResponse.json()
			throw new Error(err.error.message)
		}

        
		return new Response(chatResponse.body, {
			headers: {
				'Content-Type': 'text/event-stream'
			}
		})
	} catch (err) {
		console.error(err)
		return json({ error: 'There was an error processing your request' }, { status: 500 })
	}
}


import type { IncomingMessage, ServerResponse } from 'http';

export async function get_chat_sse(req: IncomingMessage, res: ServerResponse) {
  try {
    // Parse the JSON request body
    const chunks = [];
    for await (const chunk of req) {
      chunks.push(chunk);
    }
    const requestData = JSON.parse(Buffer.concat(chunks).toString());

    // Process the chat request
    const chatResponse = await processChatRequest(requestData);

    // Set up the SSE response
    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
    });

    // Send the chat response as an SSE event
    res.write(`data: ${JSON.stringify(chatResponse)}\n\n`);
  } catch (err) {
    console.error(err);
    res.writeHead(500);
    res.write(JSON.stringify({ error: 'There was an error processing your request' }));
  } finally {
    res.end();
  }
}
