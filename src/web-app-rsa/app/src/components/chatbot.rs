use crate::components::chat_state::{chat_completion, ChatMessage, ChatState};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_location;

#[component]
pub fn Chatbot() -> impl IntoView {
    let state = ChatState::new();
    let messages = RwSignal::new(state.messages);
    let thinking = RwSignal::new(false);
    let input_text = RwSignal::new(String::new());
    let textarea_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();
    let scroll_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let missing_config = RwSignal::new(false);
    let location = use_location();
    let close_href = move || location.pathname.get();

    let on_submit = move |_| {
        let text = input_text.get_untracked();
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() || thinking.get_untracked() {
            return;
        }
        input_text.set(String::new());
        thinking.set(true);

        let user_msg = ChatMessage {
            role: "user".into(),
            content: trimmed.clone(),
        };
        messages.update(|msgs| msgs.push(user_msg));

        let msgs_snapshot = messages.get_untracked();
        spawn_local(async move {
            match chat_completion(msgs_snapshot).await {
                Ok(reply) => {
                    messages.update(|msgs| {
                        msgs.push(ChatMessage {
                            role: "assistant".into(),
                            content: reply,
                        });
                    });
                }
                Err(e) => {
                    if e.0.contains("AI not configured") {
                        missing_config.set(true);
                    }
                    messages.update(|msgs| {
                        msgs.push(ChatMessage {
                            role: "assistant".into(),
                            content: "My apologies, but I encountered an unexpected error.".into(),
                        });
                    });
                }
            }
            thinking.set(false);
        });
    };

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.shift_key() && !thinking.get_untracked() {
            ev.prevent_default();
            if let Some(textarea) = textarea_ref.get() {
                input_text.set(textarea.value());
            }
            on_submit(());
        }
    };

    let scroll_to_bottom = move || {
        if let Some(el) = scroll_ref.get() {
            let _ = el.set_scroll_top(el.scroll_height());
        }
    };

    Effect::new(move |_| {
        messages.track();
        scroll_to_bottom();
    });

    Effect::new(move |_| {
        thinking.track();
        scroll_to_bottom();
    });

    view! {
        <div class="chatbot-pane">
            <a class="hide-chatbot" href=close_href title="Close"><span>"\u{2716}"</span></a>
            <div class="chatbot-chat" node_ref=scroll_ref>
                {move || {
                    if missing_config.get() {
                        view! {
                            <p class="message message-assistant">
                                <strong>"Chatbot is not configured."</strong>
                                " Set an AI provider in the config file."
                            </p>
                        }.into_any()
                    } else {
                        messages.get().into_iter()
                            .filter(|m| m.role != "system")
                            .map(|msg| {
                                let class = if msg.role == "user" { "message message-user" } else { "message message-assistant" };
                                view! {
                                    <p class=class>{msg.content.clone()}</p>
                                }.into_any()
                            })
                            .collect::<Vec<_>>()
                            .into_any()
                    }
                }}
                {move || thinking.get().then(|| view! {
                    <p class="chatbot-thinking">"Thinking..."</p>
                })}
            </div>
            <form class="chatbot-input" on:submit=move |ev| { ev.prevent_default(); on_submit(()); }>
                <textarea
                    node_ref=textarea_ref
                    placeholder="Start chatting..."
                    on:input=move |_| {
                        if let Some(textarea) = textarea_ref.get() {
                            input_text.set(textarea.value());
                        }
                    }
                    on:keydown=on_keydown
                ></textarea>
                <button type="submit" title="Send" disabled=move || thinking.get() || input_text.get().trim().is_empty()>
                    "Send"
                </button>
            </form>
        </div>
    }
}
