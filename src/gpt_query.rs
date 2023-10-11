use dotenv::dotenv;
use std::env;
use std::time::Duration;
use serde_json;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, ACCEPT};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Role {
    /// A system message, automatically sent at the start to set the tone of the model
    System,
    /// A message sent by ChatGPT
    Assistant,
    /// A message sent by the user
    User,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    /// Role of message sender
    role: Role,
    /// Actual content of the message
    content: String,
}


#[derive(Serialize)]
pub struct GPTRequest<'a> {
    /// The model to be used, currently `gpt-3.5-turbo`, but may change in future
    model: &'a str,
    messages: Vec<ChatMessage>,
    temperature: u32,
    max_tokens: u32,
    top_p: u32,
    frequency_penalty: u32,
    presence_penalty: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    message: String,
}

#[derive(Deserialize, Debug)]
struct GPTChoice {
    index: usize,
    message: ChatMessage,
}

#[derive(Deserialize, Debug)]
struct GPTResponse {
    id: String,
    object: String,
    created: usize,
    model: String,
    choices: Vec<GPTChoice>,
}


pub fn query_gpt() {
    dotenv().ok(); 


    let api_key = env::var("OPEN_AI_KEY").expect("Open ai key must be available");
    let bearer_auth = format!("Bearer {}", api_key);

    let client = Client::new();

    let prompt = format!("Explain the '{}' command in maximum of 250 words. The structure of the response should be:
                         ----
                         ... Start with a short description of the tool with maximum of 50 words. ...
                         ----
                         ... Provide a deconstruction of the command with short info and code examples 
                         ----
                         ... Provide a list of 5 tags in a comma separated list that represent this command for easier query when searching for this information.
                         ----", 
                         "nvim .");


    let content = serde_json::to_string(&GPTRequest {
        model: "gpt-3.5-turbo",
        messages: vec!(ChatMessage { 
            role: Role::User,
            content: prompt.clone(),
        }),
        temperature: 0,
        max_tokens: 299,
        top_p: 1,
        frequency_penalty: 0,
        presence_penalty: 0
    }).unwrap();

    let res = client.post("https://api.openai.com/v1/chat/completions")
        .timeout(Duration::new(120, 0))
        .header(ACCEPT, "*/*")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, &bearer_auth)
        .body(content)
        .send();

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let mess = response.text().unwrap();
                println!("res::: {:?}", mess);
                let j: GPTResponse = serde_json::from_str(&mess).unwrap();
                println!("res json:: {:?}", j);
            } else {
                let mess = response.text().unwrap();
                println!("res err::: {:?}", mess)
            }
        },
        Err(e) => {
            println!("ERROR::::::::::: {:?}", e);
        }
    }
}
