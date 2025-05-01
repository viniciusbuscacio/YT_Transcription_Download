use std::io::{self, Write};
use std::fs::{self, File};
use std::path::Path;
use anyhow::Result;
use colored::*;
use ytranscript::{YoutubeTranscript, TranscriptConfig};
use reqwest::Client;
use serde_json::Value;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<()> {
    loop {
        println!("\n{}", "YT Transcription Downloader".yellow().bold());
        println!("- Press {} to download all videos from a channel", "C".green().bold());
        println!("- Press {} to select a specific language", "L".blue().bold());
        println!("- Press {} to quit", "Q".red().bold());
        println!("{}", "Type the video URL:".cyan());
        
        let mut input = String::new();
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        
        if input.eq_ignore_ascii_case("q") {
            println!("{}", "Exiting program...".red().bold());
            break;
        } else if input.eq_ignore_ascii_case("c") {
            println!("{}", "Enter channel URL or ID:".cyan());
            let mut channel_input = String::new();
            io::stdin().read_line(&mut channel_input)?;
            
            match download_channel_transcripts(&channel_input.trim()).await {
                Ok(stats) => println!("{} {} {} {}", 
                    "Successfully downloaded".green().bold(), 
                    stats.0.to_string().green(), 
                    "out of".green(), 
                    stats.1.to_string().yellow()),
                Err(e) => println!("{}: {}", "Channel error".red().bold(), e),
            }
        } else if input.eq_ignore_ascii_case("l") {
            download_with_language_selection().await?;
        } else {
            // Download transcript with automatic language detection
            let urls = input.split_whitespace().collect::<Vec<&str>>();
            if urls.len() > 1 {
                println!("{} {} {}", "Processing".blue(), urls.len().to_string().yellow(), "URLs".blue());
                let mut success_count = 0;
                
                for (index, url) in urls.iter().enumerate() {
                    println!("\n{} {} {} {}", "URL".blue(), (index + 1).to_string().yellow(), "of".blue(), urls.len().to_string().yellow());
                    match download_transcript(url, None).await {
                        Ok(_) => {
                            println!("{}", "Transcription downloaded successfully!".green().bold());
                            success_count += 1;
                        },
                        Err(e) => println!("{}: {}", "Error".red().bold(), e),
                    }
                }
                
                println!("\n{} {} {} {}", "Completed:".blue(), success_count.to_string().green(), "of".blue(), urls.len().to_string().yellow());
            } else {
                match download_transcript(input, None).await {
                    Ok(_) => println!("{}", "Transcription downloaded successfully!".green().bold()),
                    Err(e) => println!("{}: {}", "Error".red().bold(), e),
                }
            }
        }
    }
    
    Ok(())
}

async fn download_with_language_selection() -> Result<()> {
    println!("{}", "Select language:".cyan());
    println!("1. English (en)");
    println!("2. Spanish (es)");
    println!("3. Portuguese (pt)");
    println!("4. French (fr)");
    println!("5. German (de)");
    println!("6. Italian (it)");
    println!("7. Japanese (ja)");
    println!("8. Korean (ko)");
    println!("9. Chinese (zh)");
    println!("0. Other (specify)");
    
    let mut lang_choice = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut lang_choice)?;
    
    let lang_code = match lang_choice.trim() {
        "1" => "en".to_string(),
        "2" => "es".to_string(),
        "3" => "pt".to_string(),
        "4" => "fr".to_string(),
        "5" => "de".to_string(),
        "6" => "it".to_string(),
        "7" => "ja".to_string(),
        "8" => "ko".to_string(),
        "9" => "zh".to_string(),
        "0" => {
            println!("Enter language code (e.g., ru, ar, hi):");
            let mut custom_lang = String::new();
            io::stdin().read_line(&mut custom_lang)?;
            custom_lang.trim().to_string()
        },
        _ => {
            println!("{}", "Invalid choice, using English (en)".yellow());
            "en".to_string()
        }
    };
    
    println!("{} {}", "Language selected:".green(), lang_code.green());
    println!("{}", "Enter video URL:".cyan());
    let mut video_url = String::new();
    io::stdin().read_line(&mut video_url)?;
    
    let urls = video_url.trim().split_whitespace().collect::<Vec<&str>>();
    if urls.len() > 1 {
        println!("{} {} {}", "Processing".blue(), urls.len().to_string().yellow(), "URLs".blue());
        let mut success_count = 0;
        
        for (index, url) in urls.iter().enumerate() {
            println!("\n{} {} {} {}", "URL".blue(), (index + 1).to_string().yellow(), "of".blue(), urls.len().to_string().yellow());
            match download_transcript(url, Some(lang_code.clone())).await {
                Ok(_) => {
                    println!("{}", "Transcription downloaded successfully!".green().bold());
                    success_count += 1;
                },
                Err(e) => println!("{}: {}", "Error".red().bold(), e),
            }
        }
        
        println!("\n{} {} {} {}", "Completed:".blue(), success_count.to_string().green(), "of".blue(), urls.len().to_string().yellow());
    } else {
        // Download transcript with selected language
        match download_transcript(&video_url.trim(), Some(lang_code)).await {
            Ok(_) => println!("{}", "Transcription downloaded successfully!".green().bold()),
            Err(e) => println!("{}: {}", "Error".red().bold(), e),
        }
    }
    
    Ok(())
}

async fn download_transcript(url: &str, lang_code: Option<String>) -> Result<()> {
    download_transcript_to_dir(url, lang_code, None).await
}

async fn download_transcript_to_dir(url: &str, lang_code: Option<String>, custom_dir: Option<&Path>) -> Result<()> {
    println!("{} {}", "Downloading transcript for:".blue(), url.white().underline());
    
    // Explicit language preference or default to English if nothing is specified
    let preferred_lang = lang_code.clone();
    
    if let Some(ref lang) = preferred_lang {
        println!("{} {}", "Selected language:".blue(), lang.green());
    } else {
        println!("{}", "Automatic language selection (prioritizing English)".blue());
    }
    
    // Configure the transcript request
    let mut transcript = None;
    let mut used_lang = String::new();
    
    // If a specific language was requested, try that language first
    if let Some(ref lang) = preferred_lang {
        let config = Some(TranscriptConfig {
            lang: Some(lang.clone()),
        });
        
        match YoutubeTranscript::fetch_transcript(url, config).await {
            Ok(t) => {
                if !t.is_empty() {
                    transcript = Some(t);
                    used_lang = lang.clone();
                    println!("{} {}", "Found transcript in requested language:".green(), used_lang.green());
                } else {
                    println!("{} {}", "No transcript available in".yellow(), lang.yellow());
                }
            },
            Err(e) => {
                println!("{} {}: {}", "Failed to get transcript in".yellow(), lang.yellow(), e);
            }
        }
    }
    
    // If no language was specified or if it failed, try English explicitly
    if transcript.is_none() && (preferred_lang.is_none() || preferred_lang.as_deref() != Some("en")) {
        let en_config = Some(TranscriptConfig {
            lang: Some("en".to_string()),
        });
        
        match YoutubeTranscript::fetch_transcript(url, en_config).await {
            Ok(t) => {
                if !t.is_empty() {
                    transcript = Some(t);
                    used_lang = "en".to_string();
                    println!("{}", "Found transcript in English".green());
                } else {
                    println!("{}", "No English transcript available".yellow());
                }
            },
            Err(e) => {
                println!("{}: {}", "Failed to get English transcript".yellow(), e);
            }
        }
    }
    
    // If we still don't have a transcript, try without specifying a language (last resort)
    if transcript.is_none() {
        println!("{}", "Trying to get any available transcript...".yellow());
        match YoutubeTranscript::fetch_transcript(url, None).await {
            Ok(t) => {
                if !t.is_empty() {
                    // Get the language from the first entry
                    if let Some(first_entry) = t.first() {
                        used_lang = first_entry.lang.clone();
                    } else {
                        used_lang = "unknown".to_string();
                    }
                    
                    transcript = Some(t);
                    println!("{} {}", "Found transcript in language:".green(), used_lang.green());
                } else {
                    return Err(anyhow::anyhow!("No transcript available for this video"));
                }
            },
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to find any transcript: {}", e));
            }
        }
    }
    
    // Check if we have a transcript
    if transcript.is_none() {
        return Err(anyhow::anyhow!("No transcript available for this video"));
    }
    
    let transcript = transcript.unwrap();
    
    // Extract video ID from URL or use the full URL if extraction fails
    let video_id = if url.contains("v=") {
        url.split("v=").nth(1).unwrap_or(url).split('&').next().unwrap_or(url)
    } else if url.contains("youtu.be/") {
        url.split("youtu.be/").nth(1).unwrap_or(url)
    } else {
        url // Use the full URL if extraction fails
    };
    
    println!("{} {} {}", "Found".green(), transcript.len().to_string().yellow(), "transcript segments".green());
    
    // Determine where to save
    let downloads_dir = match custom_dir {
        Some(dir) => dir.to_path_buf(),
        None => {
            let dir = Path::new("Downloads");
            if !dir.exists() {
                fs::create_dir(dir).map_err(|e| anyhow::anyhow!("Failed to create Downloads directory: {}", e))?;
                println!("{}", "Created Downloads directory".blue());
            }
            dir.to_path_buf()
        }
    };
    
    // Combine all transcript text and add timestamps
    let mut transcript_text = String::new();
    for entry in transcript {
        let minutes = (entry.offset / 60.0).floor();
        let seconds = (entry.offset % 60.0).floor();
        let timestamp = format!("[{:02}:{:02}] ", minutes, seconds);
        
        transcript_text.push_str(&timestamp);
        transcript_text.push_str(&entry.text);
        transcript_text.push('\n');
    }
    
    // Save to a file
    let filename = downloads_dir.join(format!("{}_{}_transcript.txt", video_id, used_lang));
    let mut file = File::create(&filename).map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
    file.write_all(transcript_text.as_bytes()).map_err(|e| anyhow::anyhow!("Failed to write to file: {}", e))?;
    
    println!("{} {}", "Transcription saved to:".green(), filename.display().to_string().bright_green());
    Ok(())
}

async fn download_channel_transcripts(channel_url: &str) -> Result<(usize, usize)> {
    // Extract the channel ID
    let channel_id = extract_channel_id(channel_url)?;
    println!("{} {}", "Channel ID:".blue(), channel_id.bright_blue());
    
    // Get the list of videos from the channel
    let videos = get_channel_videos(&channel_id).await?;
    
    if videos.is_empty() {
        println!("{}", "No videos found in this channel.".yellow());
        return Ok((0, 0));
    }
    
    println!("{} {} {}", "Found".green(), videos.len().to_string().yellow(), "videos".green());
    
    // Create the downloads folder for the channel
    let channel_dir = Path::new("Downloads").join(&channel_id);
    if !channel_dir.exists() {
        fs::create_dir_all(&channel_dir).map_err(|e| anyhow::anyhow!("Failed to create channel directory: {}", e))?;
        println!("{} {}", "Created directory:".blue(), channel_dir.display().to_string().bright_blue());
    }
    
    // Download transcripts for each video
    let mut success_count = 0;
    
    for (index, video) in videos.iter().enumerate() {
        println!("\n{} {} {} {}: {}", 
            "Video".blue(), 
            (index + 1).to_string().yellow(), 
            "of".blue(), 
            videos.len().to_string().yellow(),
            video.title.bright_white());
        
        let video_url = format!("https://www.youtube.com/watch?v={}", video.id);
        
        // Try to download the transcript for this video
        match download_transcript_to_dir(&video_url, None, Some(&channel_dir)).await {
            Ok(_) => {
                println!("{}", "✓ Transcription downloaded successfully!".green().bold());
                success_count += 1;
            },
            Err(e) => {
                println!("{}: {}", "✗ Error".red().bold(), e);
            }
        }
        
        // Small pause between requests to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("\n{} {} {} {}", 
        "Completed:".blue(), 
        success_count.to_string().green(), 
        "of".blue(), 
        videos.len().to_string().yellow());
    
    Ok((success_count, videos.len()))
}

// Structure to store video information
#[derive(Debug, Clone)]
struct VideoInfo {
    id: String,
    title: String,
}

async fn get_channel_videos(channel_id: &str) -> Result<Vec<VideoInfo>> {
    println!("{}", "Fetching videos from channel...".blue());
    
    let client = Client::new();
    let mut videos = Vec::new();
    let max_pages = 10; // Maximum number of pages to fetch (each page typically has ~30 videos)
    
    // Build the channel URL
    let url = if channel_id.contains("channel/") || channel_id.len() == 24 {
        format!("https://www.youtube.com/channel/{}/videos", channel_id)
    } else if channel_id.starts_with('@') {
        format!("https://www.youtube.com/{}/videos", channel_id)
    } else {
        format!("https://www.youtube.com/@{}/videos", channel_id)
    };
    
    println!("{} {}", "Requesting:".blue(), url.bright_blue());
    
    let response = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36")
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to request channel page: {}", e))?;
    
    let html = response.text().await
        .map_err(|e| anyhow::anyhow!("Failed to get HTML content: {}", e))?;
    
    // Extract API key for further requests
    let api_key_regex = Regex::new(r#"INNERTUBE_API_KEY":"([^"]+)"#).unwrap();
    let api_key = if let Some(captures) = api_key_regex.captures(&html) {
        if let Some(key_match) = captures.get(1) {
            key_match.as_str().to_string()
        } else {
            // Fallback API key that sometimes works
            "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8".to_string()
        }
    } else {
        "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8".to_string()
    };
    
    println!("{} {}", "Extracted API key:".blue(), api_key.bright_blue());
    
    // Extract initial videos and continuation token
    let json_regex = Regex::new(r#"var ytInitialData = (.+?);</script>"#).unwrap();
    let mut continuation_token = None;
    
    if let Some(captures) = json_regex.captures(&html) {
        if let Some(json_str) = captures.get(1) {
            match serde_json::from_str::<Value>(json_str.as_str()) {
                Ok(json_data) => {
                    // Extract initial videos
                    extract_videos_from_json(&json_data, &mut videos);
                    
                    // Extract continuation token
                    continuation_token = extract_continuation_token(&json_data);
                },
                Err(e) => {
                    println!("{}: {}", "Error parsing JSON".red(), e);
                }
            }
        }
    }
    
    // If no videos found, try regex fallback
    if videos.is_empty() {
        println!("{}", "No videos found using JSON method, trying regex fallback...".yellow());
        extract_videos_with_regex(&html, &mut videos);
    }
    
    // Use continuation token to fetch more videos
    let mut page_count = 1;
    
    while let Some(token) = continuation_token {
        if page_count >= max_pages {
            println!("{} {}", "Reached maximum number of pages:".yellow(), max_pages.to_string().yellow());
            break;
        }
        
        println!("{} {}", "Fetching more videos with continuation token (page".blue(), format!("{}/{})", page_count + 1, max_pages).blue());
        
        // Sleep to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Request next batch of videos
        match fetch_continuation_videos(&client, &api_key, &token).await {
            Ok((new_videos, new_token)) => {
                let video_count = new_videos.len();
                videos.extend(new_videos);
                continuation_token = new_token;
                
                println!("{} {} {}", "Found".green(), video_count.to_string().yellow(), "more videos".green());
                
                // Break if no new videos or no continuation token
                if video_count == 0 || continuation_token.is_none() {
                    println!("{}", "No more videos to fetch".yellow());
                    break;
                }
                
                page_count += 1;
            },
            Err(e) => {
                println!("{}: {}", "Error fetching continuation".red(), e);
                break;
            }
        }
    }
    
    println!("{} {} {}", "Found".green(), videos.len().to_string().yellow(), "videos in total".green());
    
    Ok(videos)
}

fn extract_videos_from_json(json: &Value, videos: &mut Vec<VideoInfo>) {
    // Extract videos from richGridRenderer format
    if let Some(contents) = json
        .get("contents")
        .and_then(|c| c.get("twoColumnBrowseResultsRenderer"))
        .and_then(|r| r.get("tabs"))
        .and_then(|t| t.as_array())
        .and_then(|a| a.iter().find(|tab| 
            tab.get("tabRenderer")
                .and_then(|r| r.get("title"))
                .and_then(|t| t.as_str())
                .map_or(false, |s| s == "Videos")))
        .and_then(|t| t.get("tabRenderer"))
        .and_then(|r| r.get("content"))
        .and_then(|c| c.get("richGridRenderer"))
        .and_then(|g| g.get("contents"))
        .and_then(|c| c.as_array()) 
    {
        for item in contents {
            process_video_item(item, videos);
        }
    }
    
    // Also check continuationItems if present
    if let Some(continuation_items) = json
        .get("onResponseReceivedActions")
        .and_then(|a| a.as_array())
        .and_then(|actions| actions.first())
        .and_then(|action| action.get("appendContinuationItemsAction"))
        .and_then(|a| a.get("continuationItems"))
        .and_then(|c| c.as_array())
    {
        for item in continuation_items {
            process_video_item(item, videos);
        }
    }
}

fn process_video_item(item: &Value, videos: &mut Vec<VideoInfo>) {
    // Process regular video item
    if let Some(renderer) = item.get("richItemRenderer")
        .and_then(|r| r.get("content"))
        .and_then(|c| c.get("videoRenderer")) 
    {
        extract_video_from_renderer(renderer, videos);
    }
    
    // Process grid video item (alternative format)
    if let Some(renderer) = item.get("gridVideoRenderer") {
        extract_video_from_renderer(renderer, videos);
    }
    
    // Process video item from continuation
    if let Some(renderer) = item.get("videoRenderer") {
        extract_video_from_renderer(renderer, videos);
    }
}

fn extract_video_from_renderer(renderer: &Value, videos: &mut Vec<VideoInfo>) {
    let video_id = renderer.get("videoId").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    let title = renderer
        .get("title")
        .and_then(|t| t.get("runs"))
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .and_then(|r| r.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    if let Some(id) = video_id {
        if !videos.iter().any(|v| v.id == id) {  // Check for duplicates
            videos.push(VideoInfo { id, title });
        }
    }
}

fn extract_continuation_token(json: &Value) -> Option<String> {
    println!("{}", "Searching for continuation token...".blue());
    
    // Debugging: Save the JSON to analyze structure
    let _ = fs::write("continuation_debug.json", serde_json::to_string_pretty(json).unwrap_or_default());
    
    // Try to find token in tabs structure (first method)
    let token = json
        .get("contents")
        .and_then(|c| c.get("twoColumnBrowseResultsRenderer"))
        .and_then(|r| r.get("tabs"))
        .and_then(|t| t.as_array())
        .and_then(|a| a.iter().find(|tab| 
            tab.get("tabRenderer")
                .and_then(|r| r.get("title"))
                .and_then(|t| t.as_str())
                .map_or(false, |s| s == "Videos")))
        .and_then(|t| t.get("tabRenderer"))
        .and_then(|r| r.get("content"))
        .and_then(|c| c.get("richGridRenderer"))
        .and_then(|g| g.get("contents"))
        .and_then(|c| c.as_array())
        .and_then(|a| a.iter().find(|item| item.get("continuationItemRenderer").is_some()))
        .and_then(|i| i.get("continuationItemRenderer"))
        .and_then(|r| r.get("continuationEndpoint"))
        .and_then(|e| e.get("continuationCommand"))
        .and_then(|c| c.get("token"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string());
    
    if let Some(ref t) = token {
        println!("{} {} {}", "Found token (method 1):".green(), &t[..20].bright_green(), "...".bright_green());
        return token;
    }
    
    println!("{}", "Method 1 failed, trying method 2...".yellow());
    
    // Try alternate location for continuation token
    let token2 = json
        .get("onResponseReceivedActions")
        .and_then(|a| a.as_array())
        .and_then(|actions| actions.first())
        .and_then(|action| action.get("appendContinuationItemsAction"))
        .and_then(|a| a.get("continuationItems"))
        .and_then(|c| c.as_array())
        .and_then(|a| a.iter().find(|item| item.get("continuationItemRenderer").is_some()))
        .and_then(|i| i.get("continuationItemRenderer"))
        .and_then(|r| r.get("continuationEndpoint"))
        .and_then(|e| e.get("continuationCommand"))
        .and_then(|c| c.get("token"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string());
    
    if let Some(ref t) = token2 {
        println!("{} {} {}", "Found token (method 2):".green(), &t[..20].bright_green(), "...".bright_green());
        return token2;
    }
    
    println!("{}", "Method 2 failed, trying method 3 (direct search)...".yellow());
    
    // Method 3: Direct regex search for continuation token pattern in the JSON
    let json_str = serde_json::to_string(json).unwrap_or_default();
    let token_regex = Regex::new(r#""continuationCommand":\s*\{\s*"token":\s*"([^"]+)"#).unwrap();
    
    if let Some(captures) = token_regex.captures(&json_str) {
        if let Some(token_match) = captures.get(1) {
            let token_str = token_match.as_str().to_string();
            println!("{} {} {}", "Found token (method 3):".green(), &token_str[..20].bright_green(), "...".bright_green());
            return Some(token_str);
        }
    }
    
    println!("{}", "No continuation token found in any location".red());
    None
}

async fn fetch_continuation_videos(client: &Client, api_key: &str, continuation_token: &str) -> Result<(Vec<VideoInfo>, Option<String>)> {
    let mut new_videos = Vec::new();
    
    println!("{} {}", "Requesting more videos with token:".blue(), continuation_token[..20].bright_blue());
    
    // Create JSON payload for continuation request
    let client_version = "2.20240318.01.00"; // Update this periodically
    
    let payload = serde_json::json!({
        "context": {
            "client": {
                "hl": "en",
                "gl": "US",
                "clientName": "WEB",
                "clientVersion": client_version,
                "utcOffsetMinutes": 0
            },
            "user": {
                "lockedSafetyMode": false
            },
            "request": {
                "useSsl": true,
                "internalExperimentFlags": [],
                "consistencyTokenJars": []
            }
        },
        "continuation": continuation_token
    });
    
    // YouTube InnerTube API URL
    let api_url = format!("https://www.youtube.com/youtubei/v1/browse?key={}", api_key);
    
    println!("{} {}", "Sending request to:".blue(), api_url.bright_blue());
    println!("{}", "Payload preview:".blue());
    println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default().bright_white());
    
    // Make request to YouTube API
    let response = client.post(&api_url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Origin", "https://www.youtube.com")
        .header("Referer", "https://www.youtube.com/")
        .body(payload.to_string())
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to request continuation data: {}", e))?;
    
    println!("{} {}", "Response status:".blue(), response.status().to_string().bright_blue());
    
    // Save raw response for debugging
    let response_text = response.text().await
        .map_err(|e| anyhow::anyhow!("Failed to get response text: {}", e))?;
    
    let debug_file = format!("continuation_response_{}.json", continuation_token[..10].replace("/", "_"));
    let _ = fs::write(&debug_file, &response_text);
    println!("{} {}", "Saved response to:".yellow(), debug_file);
    
    // Parse response
    let json_data: Value = serde_json::from_str(&response_text)
        .map_err(|e| anyhow::anyhow!("Failed to parse continuation JSON: {}", e))?;
    
    // Extract videos from the continuation response
    extract_videos_from_json(&json_data, &mut new_videos);
    
    println!("{} {} {}", "Extracted".green(), new_videos.len().to_string().yellow(), "new videos".green());
    
    // Extract next continuation token
    let next_continuation = extract_continuation_token(&json_data);
    
    if let Some(ref token) = next_continuation {
        println!("{} {} {}", "Found next token:".green(), &token[..20].bright_green(), "...".bright_green());
    } else {
        println!("{}", "No next continuation token found".yellow());
    }
    
    Ok((new_videos, next_continuation))
}

fn extract_videos_with_regex(html: &str, videos: &mut Vec<VideoInfo>) {
    // Specific regex for videoId in video data
    let video_regex = Regex::new(r#"videoId":"([a-zA-Z0-9_-]{11})","thumbnail"#).unwrap();
    
    let mut video_ids = std::collections::HashSet::new();
    
    for cap in video_regex.captures_iter(html) {
        if let Some(id_match) = cap.get(1) {
            let video_id = id_match.as_str().to_string();
            
            if !video_ids.contains(&video_id) {
                video_ids.insert(video_id.clone());
                
                // Try to find a title
                let title_pattern = format!(r#"videoId":"{}".*?"text":"([^"]+)"#, video_id);
                let title_regex = Regex::new(&title_pattern).unwrap();
                
                let title = if let Some(title_cap) = title_regex.captures(html) {
                    if let Some(title_match) = title_cap.get(1) {
                        title_match.as_str().to_string()
                    } else {
                        format!("Video {}", video_id)
                    }
                } else {
                    format!("Video {}", video_id)
                };
                
                videos.push(VideoInfo { id: video_id, title });
            }
        }
    }
    
    // If we still haven't found anything, try an even more generic regex
    if videos.is_empty() {
        let generic_regex = Regex::new(r#"watch\?v=([a-zA-Z0-9_-]{11})"#).unwrap();
        
        for cap in generic_regex.captures_iter(html) {
            if let Some(id_match) = cap.get(1) {
                let video_id = id_match.as_str().to_string();
                
                if !video_ids.contains(&video_id) {
                    video_ids.insert(video_id.clone());
                    videos.push(VideoInfo { 
                        id: video_id.clone(), 
                        title: format!("Video {}", video_id) 
                    });
                }
            }
        }
    }
}

fn extract_channel_id(url: &str) -> Result<String> {
    // Check if it's already a channel ID
    let channel_id_regex = Regex::new(r"^[a-zA-Z0-9_-]{24}$").unwrap();
    if channel_id_regex.is_match(url) {
        return Ok(url.to_string());
    }
    
    // If the URL already includes @ or c/, use it directly
    if url.contains("/@") || url.contains("/c/") || url.contains("/channel/") {
        // Extract the channel identifier from the URL
        let parts: Vec<&str> = url.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "@" || *part == "c" || *part == "channel" {
                if i + 1 < parts.len() {
                    return Ok(parts[i + 1].to_string());
                }
            } else if part.starts_with('@') {
                return Ok(part.to_string());
            }
        }
    }
    
    // Handle URLs in various formats
    if url.contains("youtube.com") {
        // Different URL formats for channels
        let channel_patterns = [
            r"youtube\.com/(?:channel|c|user)/([^/\s?&]+)",
            r"youtube\.com/@([^/\s?&]+)",
        ];
        
        for pattern in channel_patterns {
            let regex = Regex::new(pattern).unwrap();
            if let Some(captures) = regex.captures(url) {
                if let Some(id_match) = captures.get(1) {
                    return Ok(id_match.as_str().to_string());
                }
            }
        }
    }
    
    // If it looks like a username without the full URL
    if !url.contains('/') && !url.contains('.') {
        if url.starts_with('@') {
            return Ok(url.to_string());
        } else {
            return Ok(format!("@{}", url));
        }
    }
    
    Err(anyhow::anyhow!("Could not extract channel ID from URL. Use format: youtube.com/channel/CHANNEL_ID or youtube.com/@username"))
}