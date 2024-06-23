use aitelebot::consts::{DEFAULT_SYSTEM_MOCK, MODEL_MIXTRAL};
use anyhow::Result;
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage, MessageRole},
    Ollama,
};
use teloxide::{
    requests::Requester,
    types::Message,
    Bot,
    prelude::*,
    utils::command::BotCommands,
};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let system_msg = ChatMessage::new(MessageRole::System, DEFAULT_SYSTEM_MOCK.to_string());
    let thread_msgs: Arc<Mutex<Vec<ChatMessage>>> = Arc::new(Mutex::new(vec![system_msg]));

    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let bot = Bot::from_env();
    Command::repl(bot.clone(), move |bot, msg, cmd| {
        let mut ollama = Ollama::new_with_history("http://4.145.104.5".to_string(), 11434, 3);
        let thread_msgs = Arc::clone(&thread_msgs);
        async move {
            answer(&mut ollama, bot, msg, cmd, &thread_msgs).await
        }
    })
    .await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "some text what you want to ask AI.")]
    Ask(String),
}

async fn answer(
    ollama: &mut Ollama,
    bot: Bot,
    msg: Message,
    cmd: Command,
    thread_msgs: &Arc<Mutex<Vec<ChatMessage>>>,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Ask(sometext) => {
            match get_ollama_response(ollama, sometext, msg.chat.id.to_string(), thread_msgs).await {
                Ok(response) => {
                    bot.send_message(msg.chat.id, response).await?;
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, format!("Error: {}", e)).await?;
                }
            }
        }
    };

    Ok(())
}

async fn get_ollama_response(
    ollama: &mut Ollama,
    input: String,
    id: String,
    thread_msgs: &Arc<Mutex<Vec<ChatMessage>>>,
) -> Result<String> {
    let prompt_msg = ChatMessage::new(MessageRole::User, input);

    {
        let mut msgs = thread_msgs.lock().unwrap();
        msgs.push(prompt_msg);
        println!("{:?}", *msgs);
    }   

    let chat_req = {
        let msgs = thread_msgs.lock().unwrap();
        ChatMessageRequest::new(MODEL_MIXTRAL.to_string(), msgs.clone())
    };

    let msg_content = run_chat_req(ollama, chat_req).await?.unwrap();

    println!("-----------------------------------------------------------------");

    {
        let mut msgs = thread_msgs.lock().unwrap();
        let bot_msg = ChatMessage::new(MessageRole::Assistant, msg_content.clone());
        msgs.push(bot_msg);
        println!("{:?}", *msgs);
    }

    Ok(msg_content)
}

pub async fn run_chat_req(
    ollama: &Ollama,
    chat_req: ChatMessageRequest,
) -> Result<Option<String>> {
    let mut stream = ollama.send_chat_messages_stream(chat_req).await?;

    let mut stdout = tokio::io::stdout();
    let mut char_count = 0;
    let mut current_asst_msg_elems: Vec<String> = Vec::new();

    while let Some(res) = stream.next().await {
        let res = res.map_err(|_| "stream.next error").unwrap();

        if let Some(msg) = res.message {
            let msg_content = msg.content;

            // Poor man's wrapping
            char_count += msg_content.len();
            if char_count > 80 {
                stdout.write_all(b"\n").await?;
                char_count = 0;
            }

            // Write output
            stdout.write_all(msg_content.as_bytes()).await?;
            stdout.flush().await?;

            current_asst_msg_elems.push(msg_content);
        }

        if let Some(_final_res) = res.final_data {
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;

            let asst_content = current_asst_msg_elems.join("");
            return Ok(Some(asst_content));
        }
    }

    // new line
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;

    Ok(None)
}
