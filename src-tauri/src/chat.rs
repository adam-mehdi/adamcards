

use tauri::{Window, command};
use serde::Serialize;

use std::error::Error;

use async_openai::{
    types::{
        ChatCompletionRequestMessageArgs, 
        ChatChoiceDelta, 
        CreateChatCompletionRequestArgs, 
        Role,
        CreateModerationRequestArgs, 
        TextModerationModel
    },
    Client,
};

use futures::StreamExt;

#[derive(Serialize, Clone)]
struct ChatGptResponsePayload {
    choices: Vec<ChatChoiceDelta>, // contains output of model
}

#[command]
pub async fn send_gpt_request(
    api_key: String,
    messages: Vec<String>,
    system_prompt: String,
    max_tokens: i32,
    window: Window,
    card_id: Option<i32>
) -> Result<String, String> {


    // Check moderations
    let prompt = messages.last().expect("messages empty").clone();
    
    if let Err(_) = check_query_with_moderation(prompt, api_key.clone()).await {
        window.emit("FLAGGED_AT_MODERATIONS", format!("{}", "Query flagged by OpenAI"))
            .expect("failed to emit modifications error");
        return Ok("moderation complete".to_string());
    }   
    
    // Set up the client
    let client = Client::new().with_api_key(api_key);

    // Create the system message
    let system_message = ChatCompletionRequestMessageArgs::default()
        .role(Role::System)
        .content(system_prompt)
        .build()
        .expect("failed to build system prompt");


    // Convert user messages to ChatMessages
    let user_messages = messages
        .into_iter()
        .enumerate()
        .map(|(index, content)| {
            let role = if index % 2 == 0 {
                Role::User
            } else {
                Role::Assistant
            };
            ChatCompletionRequestMessageArgs::default()
                .role(role)
                .content(content)
                .build()
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to build context and prompt");


    // Create the chat request
    let chat_request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .messages([&[system_message], user_messages.as_slice()].concat())
        .max_tokens(max_tokens as u16)
        .stream(true)
        .build()
        .expect("failed to send ChatCompletionRequest");

    // Send the request and process the response
    let mut chat_stream = 
        client.chat().create_stream(chat_request).await.expect("failed to create stream");

    while let Some(result) = chat_stream.next().await {
        match result {
            Ok(chat_message) => {
                let payload = ChatGptResponsePayload {
                    choices: chat_message.choices,
                };
                let emission = if let Some(id) = card_id {
                    format!("CHATGPT_RESPONSE_{}", id)
                } else {
                    String::from("CHATGPT_RESPONSE")
                };
                window.emit(&emission, Some(payload)).unwrap();
            }
            Err(e) => return Err(format!("Error: {}", e)),
        }
    }


    Ok(String::from("Request sent"))
}



async fn check_query_with_moderation(message: String, api_key: String) -> Result<(), Box<dyn Error>> {
    let client = Client::new().with_api_key(api_key);

    let request = CreateModerationRequestArgs::default()
        .input(&message)
        .model(TextModerationModel::Latest)
        .build()?;

    let response = client.moderations().create(request).await?;

    if response.results.first().expect("moderations empty").flagged {
        return Err("Query flagged by OpenAI".into());
    }

    Ok(())
}
