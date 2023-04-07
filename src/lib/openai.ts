// src/lib/openai.ts

// import { Configuration, OpenAIApi } from 'openai';

// const configuration = new Configuration({
//   apiKey: process.env.OPENAI_API_KEY,
// });

// const openai = new OpenAIApi(configuration);

// export async function sendMessageGPT(prompt: string): Promise<string> {
//   try {
//     const completion = await openai.createCompletion({
//       model: 'gpt-3.5-turbo',
//       prompt: prompt,
//     });
//     return completion.data.choices[0].text!;
//   } catch (error: unknown) {
//     if (error instanceof Error && 'response' in error) {
//       console.log(error.response);
//     } else {
//       console.log((error as Error).message);
//     }
//     return '';
//   }
// }
