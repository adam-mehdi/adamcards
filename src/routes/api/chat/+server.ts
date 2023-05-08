import type { CreateChatCompletionRequest, ChatCompletionRequestMessage } from 'openai'
import type { RequestHandler } from './$types'
import { getTokens } from '$lib/tokenizer'
import { json } from '@sveltejs/kit'


type ChatRequestData = {
    messages: ChatCompletionRequestMessage[];
    systemPrompt: string;
    maxTokens?: number;
    apiKey: string
};

export const POST: RequestHandler = async ({ request }) => {
    
    const requestData = await request.json();
    const OPENAI_API_KEY = requestData.apiKey
    const reqMessages: ChatCompletionRequestMessage[] = requestData.messages;

    if (!reqMessages) {
        throw new Error('no messages provided');
    }

    let tokenCount = 0;

    reqMessages.forEach((msg) => {
        const tokens = getTokens(msg.content);
        tokenCount += tokens;
    });

    const moderationRes = await fetch('https://api.openai.com/v1/moderations', {
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${OPENAI_API_KEY}`,
        },
        method: 'POST',
        body: JSON.stringify({
            input: reqMessages[reqMessages.length - 1].content,
        }),
    });

    if (!moderationRes.ok) {
    const err = await moderationRes.json();
        throw new Error(err.error.message);
    }

    const moderationData = await moderationRes.json();
    const [results] = moderationData.results;

    if (results.flagged) {
        throw new Error('Query flagged by openai');
    }

    const prompt = requestData.systemPrompt;
    tokenCount += getTokens(prompt);

    if (tokenCount >= 1000) {
        throw new Error('Query too large');
    }

    const messages: ChatCompletionRequestMessage[] = [
        { role: 'system', content: prompt }, ...reqMessages, 
    ];

    const max_tokens = requestData.maxTokens ? requestData.maxTokens : 150;
    const chatRequestOpts: CreateChatCompletionRequest = {
        model: 'gpt-3.5-turbo',
        messages,
        temperature: 0.5,
        stream: true,
        max_tokens,
        frequency_penalty: 1.0,
    };

    const chatResponse = await fetch('https://api.openai.com/v1/chat/completions', {
        headers: {
            Authorization: `Bearer ${OPENAI_API_KEY}`,
            'Content-Type': 'application/json',
        },
        method: 'POST',
        body: JSON.stringify(chatRequestOpts),
    });

    if (!chatResponse.ok) {
        const err = await chatResponse.json();
        throw new Error(err.error.message);
    }

    return chatResponse;


}
