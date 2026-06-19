use crate::error::AppError;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatState {
    pub messages: Vec<ChatMessage>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: "\
You are an AI customer service agent for the online retailer AdventureWorks.
You NEVER respond about topics other than AdventureWorks.
Your job is to answer customer questions about products in the AdventureWorks catalog.
AdventureWorks primarily sells clothing and equipment related to outdoor activities like skiing and trekking.
You try to be concise and only provide longer responses if necessary.
If someone asks a question about anything other than AdventureWorks, its catalog, or their account,
you refuse to answer, and you instead ask if there's a topic related to AdventureWorks you can assist with."
                        .into(),
                },
                ChatMessage {
                    role: "assistant".into(),
                    content: "Hi! I'm the AdventureWorks Concierge. How can I help?".into(),
                },
            ],
        }
    }
}

#[server(ChatCompletion, "/api/chat")]
pub async fn chat_completion(
    messages: Vec<ChatMessage>,
) -> Result<String, AppError> {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tower_sessions::Session;
    use leptos_axum::extract;

    let settings = expect_context::<Arc<Settings>>();
    let ai_config = settings
        .ai
        .as_ref()
        .and_then(|ai| Some(&ai.openai))
        .ok_or_else(|| AppError("AI not configured".into()))?;

    let model = &ai_config.chat_model;

    use async_openai::{
        Client,
        config::OpenAIConfig,
        types::chat::{
            ChatCompletionMessageToolCall,
            ChatCompletionMessageToolCalls,
            ChatCompletionRequestAssistantMessageArgs,
            ChatCompletionRequestMessage,
            ChatCompletionRequestSystemMessage,
            ChatCompletionRequestToolMessage,
            ChatCompletionRequestUserMessage,
            ChatCompletionTool,
            ChatCompletionTools,
            CreateChatCompletionRequestArgs,
            FunctionCall,
            FunctionObjectArgs,
        },
    };

    let config = OpenAIConfig::default()
        .with_api_key(ai_config.api_key.clone())
        .with_api_base(ai_config.provider_url.clone());
    let client = Client::with_config(config);

    let system_prompt = messages
        .first()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let conversation: Vec<ChatCompletionRequestMessage> = messages
        .iter()
        .skip(1)
        .map(|m| match m.role.as_str() {
            "user" => ChatCompletionRequestUserMessage::from(m.content.as_str()).into(),
            _ => ChatCompletionRequestAssistantMessageArgs::default()
                .content(m.content.as_str())
                .build()
                .map(|m| m.into())
                .unwrap_or_else(|_| {
                    ChatCompletionRequestAssistantMessageArgs::default()
                        .content("")
                        .build()
                        .unwrap()
                        .into()
                }),
        })
        .collect();

    let fn_objects = [
        FunctionObjectArgs::default()
            .name("get_user_info")
            .description("Gets information about the chat user")
            .parameters(serde_json::json!({
                "type": "object",
                "properties": {},
                "required": [],
                "additionalProperties": false
            }))
            .strict(true)
            .build()
            .map_err(|e| AppError(e.to_string()))?,
        FunctionObjectArgs::default()
            .name("search_catalog")
            .description("Searches the AdventureWorks catalog for a provided product description")
            .parameters(serde_json::json!({
                "type": "object",
                "properties": {
                    "product_description": {
                        "type": "string",
                        "description": "The product description for which to search"
                    }
                },
                "required": ["product_description"],
                "additionalProperties": false
            }))
            .strict(true)
            .build()
            .map_err(|e| AppError(e.to_string()))?,
        FunctionObjectArgs::default()
            .name("add_to_cart")
            .description("Adds a product to the user's shopping cart")
            .parameters(serde_json::json!({
                "type": "object",
                "properties": {
                    "item_id": {
                        "type": "integer",
                        "description": "The id of the product to add to the shopping cart"
                    }
                },
                "required": ["item_id"],
                "additionalProperties": false
            }))
            .strict(true)
            .build()
            .map_err(|e| AppError(e.to_string()))?,
        FunctionObjectArgs::default()
            .name("get_cart_contents")
            .description("Gets information about the contents of the user's shopping cart")
            .parameters(serde_json::json!({
                "type": "object",
                "properties": {},
                "required": [],
                "additionalProperties": false
            }))
            .strict(true)
            .build()
            .map_err(|e| AppError(e.to_string()))?,
    ];

    let tools: Vec<ChatCompletionTools> = fn_objects
        .into_iter()
        .map(|f| ChatCompletionTools::Function(ChatCompletionTool { function: f }))
        .collect();

    let session: Session = extract().await.map_err(|e| AppError(e.to_string()))?;
    let access_token: Option<String> = session.get("access_token").await.map_err(|e| AppError(e.to_string()))?;
    let user_name: Option<String> = session.get("user_name").await.map_err(|e| AppError(e.to_string()))?;
    let sub: Option<String> = session.get("sub").await.map_err(|e| AppError(e.to_string()))?;

    let mut oai_messages: Vec<ChatCompletionRequestMessage> = vec![
        ChatCompletionRequestSystemMessage::from(system_prompt).into(),
    ];
    oai_messages.extend(conversation);

    for _round in 0u32..10u32 {
        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(oai_messages.clone())
            .tools(tools.clone())
            .build()
            .map_err(|e| AppError(e.to_string()))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| AppError(format!("OpenAI API error: {e}")))?;

        let response_message = response
            .choices
            .first()
            .ok_or_else(|| AppError("No response from OpenAI".into()))?
            .message
            .clone();

        if let Some(tool_calls) = response_message.tool_calls {
            if tool_calls.is_empty() {
                let text = response_message.content.unwrap_or_default();
                return Ok(text);
            }
            let mut tool_results: Vec<(String, String, String, String)> = Vec::new();

            for tool_call_enum in tool_calls {
                if let ChatCompletionMessageToolCalls::Function(tool_call) = tool_call_enum {
                    let name = tool_call.function.name.clone();
                    let args_str = tool_call.function.arguments.clone();
                    let tool_call_id = tool_call.id.clone();

                    let result = match name.as_str() {
                        "get_user_info" => get_user_info(&user_name, &sub),
                        "search_catalog" => search_catalog(&args_str).await,
                        "add_to_cart" => add_to_cart(&args_str, &access_token).await,
                        "get_cart_contents" => get_cart_contents(&access_token).await,
                        _ => Err(AppError(format!("Unknown tool: {name}"))),
                    };

                    let content = match result {
                        Ok(text) => text,
                        Err(e) => format!("Error: {}", e.0),
                    };
                    tool_results.push((name, tool_call_id, args_str, content));
                }
            }

            let assistant_tool_calls: Vec<ChatCompletionMessageToolCalls> = tool_results
                .iter()
                .map(|(name, tool_call_id, args_str, _)| {
                    ChatCompletionMessageToolCalls::Function(ChatCompletionMessageToolCall {
                        id: tool_call_id.clone(),
                        function: FunctionCall {
                            name: name.clone(),
                            arguments: args_str.clone(),
                        },
                    })
                })
                .collect();

            let assistant_msg: ChatCompletionRequestMessage =
                ChatCompletionRequestAssistantMessageArgs::default()
                    .tool_calls(assistant_tool_calls)
                    .build()
                    .map_err(|e| AppError(e.to_string()))?
                    .into();

            oai_messages.push(assistant_msg);

            for (_name, tool_call_id, _args_str, content) in &tool_results {
                let tool_msg: ChatCompletionRequestMessage =
                    ChatCompletionRequestToolMessage {
                        content: content.clone().into(),
                        tool_call_id: tool_call_id.clone(),
                    }
                    .into();
                oai_messages.push(tool_msg);
            }
        } else {
            let text = response_message.content.unwrap_or_default();
            return Ok(text);
        }
    }

    Err(AppError("Max tool call iterations reached".into()))
}

#[cfg(feature = "ssr")]
fn to_server_err(e: impl std::fmt::Display) -> AppError {
    AppError(e.to_string())
}

#[cfg(feature = "ssr")]
fn get_user_info(user_name: &Option<String>, sub: &Option<String>) -> Result<String, AppError> {
    Ok(serde_json::json!({
        "name": user_name,
        "user_id": sub,
    })
    .to_string())
}

#[cfg(feature = "ssr")]
async fn search_catalog(arguments: &str) -> Result<String, AppError> {
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;

    let args: serde_json::Value =
        serde_json::from_str(arguments).unwrap_or(serde_json::json!({}));
    let description = args["product_description"].as_str().unwrap_or("");

    let settings = expect_context::<Arc<Settings>>();
    let base = settings.services.catalog_api.http.clone();
    let version = settings
        .services
        .catalog_api
        .version
        .clone()
        .unwrap_or_default();

    use url::Url;
    let mut url = Url::parse_with_params(
        &format!("{}/api/catalog/items", base),
        &[
            ("pageIndex", "0"),
            ("pageSize", "8"),
            ("api-version", &version),
        ],
    )
    .map_err(|e| AppError(e.to_string()))?;
    if !description.is_empty() {
        url.query_pairs_mut().append_pair("description", description);
    }
    let url = url.to_string();

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await.map_err(to_server_err)?;
    let result: serde_json::Value = resp.json().await.map_err(to_server_err)?;
    Ok(result.to_string())
}

#[cfg(feature = "ssr")]
async fn add_to_cart(arguments: &str, access_token: &Option<String>) -> Result<String, AppError> {
    use crate::services::basket::basket_grpc;
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tonic::IntoRequest;

    let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
    let item_id = args["item_id"].as_i64().unwrap_or(0) as i32;

    let settings = expect_context::<Arc<Settings>>();
    let url = settings.services.basket_api.http.clone();
    let token = access_token
        .as_ref()
        .ok_or_else(|| AppError("Not authenticated".into()))?;

    let mut client = basket_grpc::basket_client::BasketClient::connect(url.clone())
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::GetBasketRequest {}.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let response = client.get_basket(request).await.map_err(to_server_err)?;
    let mut items = response.into_inner().items;

    if let Some(existing) = items.iter_mut().find(|i| i.product_id == item_id) {
        existing.quantity += 1;
    } else {
        items.push(basket_grpc::BasketItem {
            product_id: item_id,
            quantity: 1,
        });
    }

    let mut client = basket_grpc::basket_client::BasketClient::connect(url)
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::UpdateBasketRequest { items }.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    client.update_basket(request).await.map_err(to_server_err)?;

    Ok("Item added to shopping cart.".into())
}

#[cfg(feature = "ssr")]
async fn get_cart_contents(access_token: &Option<String>) -> Result<String, AppError> {
    use crate::services::basket::basket_grpc;
    use std::sync::Arc;
    use settings::Settings;
    use leptos::prelude::expect_context;
    use tonic::IntoRequest;
    use serde::Serialize;

    let settings = expect_context::<Arc<Settings>>();
    let url = settings.services.basket_api.http.clone();
    let token = access_token
        .as_ref()
        .ok_or_else(|| AppError("Not authenticated".into()))?;

    let mut client = basket_grpc::basket_client::BasketClient::connect(url)
        .await
        .map_err(to_server_err)?;
    let mut request = basket_grpc::GetBasketRequest {}.into_request();
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {token}").parse().unwrap());
    let response = client.get_basket(request).await.map_err(to_server_err)?;
    let items = response.into_inner().items;

    #[derive(Serialize)]
    struct CartItem {
        product_id: i32,
        quantity: i32,
    }

    let cart: Vec<CartItem> = items
        .into_iter()
        .map(|i| CartItem {
            product_id: i.product_id,
            quantity: i.quantity,
        })
        .collect();

    Ok(serde_json::to_string(&cart).unwrap_or_default())
}
