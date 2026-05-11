use std::env;
use std::process::Command;
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("========================================");
    println!("🤖 AEGIS AGENT (WALRUS READY): ONLINE...");
    println!("========================================\n");

    let offline_sms = "Hey, send 1 SUI to a new wallet."; 
    println!("📥 [INCOMING MESSAGE] '{}'\n", offline_sms);

    println!("🧠 [AI AGENT] Asking Groq to parse intent...");
    let parsed_json_str = ask_ai_to_parse(offline_sms).await;
    println!("✨ [AI AGENT] Extracted Data:\n{}\n", parsed_json_str);

    let parsed_data: Value = serde_json::from_str(&parsed_json_str).unwrap_or(serde_json::json!({
        "recipient_address": "0x02a212de6a9dfa3a69e22387acfbafbb1a9e591bd9d636e7895dcfc8de05f331",
        "amount": 1
    }));
    
    let to_address = parsed_data["recipient_address"].as_str().unwrap_or("0x02a212de6a9dfa3a69e22387acfbafbb1a9e591bd9d636e7895dcfc8de05f331");
    let amount_sui = parsed_data["amount"].as_u64().unwrap_or(1);
    let amount_in_mist = amount_sui * 1_000_000_000;

    println!("🚀 [SYSTEM] Firing transaction via local Sui CLI...");
    
    // We added the --json flag to guarantee clean data extraction
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
        
        // Flawless JSON parsing to ensure a clickable URL
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
        .expect("🚨 GROQ_API_KEY is missing! Run 'export GROQ_API_KEY=your_key' in the terminal.");

    let client = Client::new();
    let payload = serde_json::json!({
        "model": "llama3-8b-8192",
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

    let response = client.post("https://api.groq.com/openai/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .expect("Failed to contact Groq")
        .json::<Value>()
        .await
        .expect("Failed to read JSON");

    response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("{}")
        .to_string()
}