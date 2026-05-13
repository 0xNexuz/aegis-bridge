use std::env;
use std::process::Command;
use reqwest::Client;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("========================================");
    println!("🤖 AEGIS AGENT (WALRUS READY): ONLINE...");
    println!("========================================\n");

    // 1. The Simulated Offline SMS - REPLACE WITH YOUR ACTUAL ADDRESS
    let offline_sms = "Hey, send 1 SUI to 0xe073106722baf6ab028dd6ceea06a3bdd1ea6c8348cd3d8952a9054b709000cd"; 
    println!("📥 [INCOMING MESSAGE] '{}'\n", offline_sms);

    // 2. Call the AI Brain
    println!("🧠 [AI AGENT] Asking Groq to parse intent...");
    let parsed_json_str = ask_ai_to_parse(offline_sms).await;
    
    // Safety check to strip backticks if Groq ignores the JSON mode
    let clean_json = parsed_json_str
        .replace("```json", "")
        .replace("
```", "")
        .trim()
        .to_string();

    println!("✨ [AI AGENT] Extracted Data:\n{}\n", clean_json);

    // 3. Parse the data or use safety fallback
    let parsed_data: Value = serde_json::from_str(&clean_json).unwrap_or(json!({
        "recipient_address": "0xe073106722baf6ab028dd6ceea06a3bdd1ea6c8348cd3d8952a9054b709000cd",
        "amount": 1
    }));
    
    let to_address = parsed_data["recipient_address"].as_str().unwrap_or("0xe073106722baf6ab028dd6ceea06a3bdd1ea6c8348cd3d8952a9054b709000cd");
    
    // Flexible amount parsing (handles string or number from AI)
    let amount_sui = if parsed_data["amount"].is_number() {
        parsed_data["amount"].as_f64().unwrap_or(1.0)
    } else {
        parsed_data["amount"].as_str().unwrap_or("1").parse::<f64>().unwrap_or(1.0)
    };
    
    let amount_in_mist = (amount_sui * 1_000_000_000.0) as u64;

    // 4. Execution via Sui CLI
    println!("🚀 [SYSTEM] Firing transaction via local Sui CLI...");
    
    let output = Command::new("sui")
        .arg("client")
        .arg("ptb")
        .arg("--split-coins")
        .arg("gas")
        .arg(format!("[{}]", amount_in_mist))
        .arg("--assign")
        .arg("split_coin")
        .arg("--transfer-objects")
        .arg("[split_coin]")
        .arg(format!("@{}", to_address))
        .arg("--gas-budget")
        .arg("50000000")
        .arg("--json") 
        .output()
        .expect("🚨 Failed to execute Sui CLI command");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    
    if output.status.success() {
        println!("\n🎉 [SUCCESS] TRANSACTION EXECUTED!");
        
        if let Ok(json_out) = serde_json::from_str::<Value>(&stdout_str) {
            if let Some(digest) = json_out["digest"].as_str() {
                println!("🔗 CLICK TO VIEW: https://suiscan.xyz/devnet/tx/{}", digest);
            } else {
                println!("✅ Executed successfully! Hash extraction failed. Raw: {}", stdout_str);
            }
        } else {
            println!("✅ Executed successfully! Output: {:.100}...", stdout_str);
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("🚨 [ERROR] Transaction failed:\n{}", stderr);
    }
    
    println!("========================================");
    Ok(())
}

async fn ask_ai_to_parse(note: &str) -> String {
    let api_key = env::var("GROQ_API_KEY")
        .expect("🚨 GROQ_API_KEY is missing! Run '$env:GROQ_API_KEY=\"your_key\"' in the terminal.");

    let client = Client::new();
    let payload = json!({
        "model": 
        "llama-3.1-8b-instant",
        "response_format": { "type": "json_object" },
        "messages": [
            {
                "role": "system",
                "content": "You are a robotic financial parser. Extract the action, amount, and recipient. Reply ONLY with raw JSON. Example Format: {\"action\": \"transfer\", \"amount\": 1, \"recipient_address\": \"0x02a212de6a9dfa3a69e22387acfbafbb1a9e591bd9d636e7895dcfc8de05f331\"}"
            },
            {
                "role": "user",
                "content": note
            }
        ]
    });

    let res = client.post("https://api.groq.com/openai/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .expect("Failed to contact Groq");

    // NEW DEBUG BLOCK: If Groq fails, we print the exact reason
    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        println!("🔍 [DEBUG] GROQ API REJECTED REQUEST: {}", err_text);
        return "{}".to_string();
    }

    let response: Value = res.json().await.expect("Failed to read JSON");
    
    response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("{}")
        .to_string()
}