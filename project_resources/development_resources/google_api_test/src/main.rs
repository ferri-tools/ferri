use std::env;
use std::fs;
use serde::Deserialize;
use base64::Engine as _;

#[derive(Debug, Deserialize)]
struct GeminiPrediction {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    #[serde(rename = "inlineData")]
    inline_data: Option<GeminiInlineData>,
}

#[derive(Debug, Deserialize)]
struct GeminiInlineData {
    data: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiPrediction>,
}


#[tokio::main]
async fn main() {
    // 1. Get API Key from environment variable
    let api_key = match env::var("GOOGLE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: GOOGLE_API_KEY environment variable not set.");
            return;
        }
    };

    // 2. Define the model and endpoint
    let model_name = "gemini-2.5-flash-image-preview";
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model_name
    );

    // 3. Create the JSON request body
    let body = serde_json::json!({
        "contents": [{
            "parts": [{
                "text": "a photorealistic picture of a cat"
            }]
        }]
    });

    // 4. Build and send the request
    println!("--- Sending Gemini Request ---");
    println!("URL: {}", url);
    println!("----------------------------");

    let client = reqwest::Client::new();
    let response = match client.post(&url)
        .header("x-goog-api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Error sending request: {}", e);
                return;
            }
        };

    let status = response.status();
    println!("Response Status: {}", status);

    if status.is_success() {
        match response.json::<GeminiResponse>().await {
            Ok(gemini_response) => {
                if let Some(prediction) = gemini_response.candidates.get(0) {
                    if let Some(part) = prediction.content.parts.get(0) {
                        if let Some(inline_data) = &part.inline_data {
                            match base64::engine::general_purpose::STANDARD.decode(&inline_data.data) {
                                Ok(image_bytes) => {
                                    match fs::write("gemini_cat.png", &image_bytes) {
                                        Ok(_) => println!("Successfully saved image to gemini_cat.png"),
                                        Err(e) => eprintln!("Error saving image: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("Error decoding base64 image data: {}", e),
                            }
                        } else {
                            eprintln!("Response did not contain image data.");
                        }
                    }
                } else {
                    eprintln!("Response did not contain any candidates.");
                }
            }
            Err(e) => {
                eprintln!("Error parsing JSON response: {}", e);
            }
        }
    } else {
        match response.text().await {
            Ok(text) => eprintln!("API Error Response:\n{}", text),
            Err(e) => eprintln!("Failed to read error response body: {}", e),
        }
    }
}
