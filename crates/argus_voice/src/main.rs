use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use vosk::{Model, Recognizer};
use std::time::{Instant, Duration};

// THE ENTITY EXTRACTOR
fn extract_dynamic_port(command: &str) -> Option<u16> {
    let mut target = command
        .replace("argus", "")
        .replace("august", "") 
        .replace("kill port", "")
        .replace("clear port", "")
        .replace("close port", "")
        .replace("terminate port", "")
        .trim()
        .to_string();

    target = target
        .replace("thousand", "zero zero zero")
        .replace("hundred", "zero zero")
        .replace("oh", "0"); // Translates the developer "oh" into a mathematical zero

    // The Master Slang Dictionary
    match target.as_str() {
        "eighty eighty" => return Some(8080),
        "eighty eighty one" => return Some(8081),
        "eighty eight" => return Some(88),
        "fifty one seventy three" => return Some(5173),
        "fifty four thirty two" => return Some(5432), // Postgres
        "thirty three 0 six" => return Some(3306),    // MySQL
        "sixty three seventy nine" => return Some(6379), // Redis
        "forty two 0 0" => return Some(4200),         // Angular
        _ => {} 
    }

    let mut digit_string = String::new();
    for word in target.split_whitespace() {
        let digit = match word {
            "zero" | "0" => "0", "one" => "1", "two" => "2", "three" => "3",
            "four" => "4", "five" => "5", "six" => "6", "seven" => "7",
            "eight" => "8", "nine" => "9",
            _ => "" 
        };
        digit_string.push_str(digit);
    }

    digit_string.parse::<u16>().ok()
}

// Reads the external JSON file and parses it into a list of Strings
fn load_grammar_file(file_path: &str) -> Vec<String> {
    let mut words = Vec::new();
    if let Ok(content) = std::fs::read_to_string(file_path) {
        // Strip the JSON brackets and quotes
        let cleaned = content.replace("[", "").replace("]", "").replace("\"", "");
        for word in cleaned.split(',') {
            let w = word.trim();
            if !w.is_empty() {
                words.push(w.to_string());
            }
        }
    } else {
        eprintln!("--> [WARNING] Could not locate discovered_grammar.json. Argus will crash.");
    }
    words
}

fn main() {
    println!("Awakening Argus...");

    let host = cpal::default_host();
    let mic = host.default_input_device().expect("No microphone detected.");
    let config = mic.default_input_config().expect("Could not read mic config.");
    
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    println!("Argus locked onto: {} ({} Hz, {} Channels)", mic.name().unwrap_or_else(|_| "Unknown".to_string()), sample_rate, channels);

    let model_path = "crates/argus_voice/model";
    let model = Model::new(model_path).expect("CRITICAL FAILURE: Model not found.");

    // --- THE DYNAMIC GRAMMAR LOCK ---
    let dynamic_grammar = load_grammar_file("crates/argus_voice/discovered_grammar.json");
    let allowed_words: Vec<&str> = dynamic_grammar.iter().map(|s| s.as_str()).collect();

    let recognizer = Arc::new(Mutex::new(
        Recognizer::new_with_grammar(&model, sample_rate, &allowed_words).unwrap()
    ));

    println!("Grammar Lock Engaged from external JSON. Background noise will be ignored.");

    let recognizer_clone = recognizer.clone();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    
    // --- THE WAKE STATE TRACKERS ---
    let mut is_awake = false;
    let mut last_wake_time = Instant::now();
    let wake_timeout = Duration::from_secs(5);

    let stream = mic.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| {
            let gain: f32 = 5.0; 
            
            let i16_data: Vec<i16> = data.chunks_exact(channels)
                .map(|chunk| {
                    let amplified = (chunk[0] * gain).clamp(-1.0, 1.0);
                    (amplified * i16::MAX as f32) as i16
                })
                .collect();
            
            let mut rec = recognizer_clone.lock().unwrap();
            
            if rec.accept_waveform(&i16_data) == vosk::DecodingState::Finalized {
                if let vosk::CompleteResult::Single(res) = rec.result() {
                    let command = res.text.trim();
                    
                    if !command.is_empty() && command != "[unk]" {
                        
                        // 1. THE WAKE TRIGGER
                        if command.contains("argus") || command.contains("august") {
                            is_awake = true;
                            last_wake_time = Instant::now();
                            println!("\n[EYE OPENED] Yes, Aranya?");
                        }

                        // 2. THE EXECUTION BLOCK
                        if is_awake && last_wake_time.elapsed() < wake_timeout {
                            
                            // Prevent triggering if you *only* said his name
                            if command != "argus" && command != "august" {
                                println!("--> Argus Executing: '{}'", command);
                                
                                let is_port_hit = command.contains("kill port") || 
                                                  command.contains("clear port") || 
                                                  command.contains("close port") || 
                                                  command.contains("terminate port");

                                if is_port_hit {
                                    if let Some(port) = extract_dynamic_port(command) {
                                        println!("--> ACTION: Initiating termination protocol for port {}...", port);
                                        argus_daemon::assassinate_port(port);
                                    } else {
                                        println!("--> [DAEMON] ERROR: I heard the command, but couldn't understand the port number.");
                                    }
                                } 
                                else if command.contains("system memory") {
                                    println!("--> ACTION: Reading telemetry...");
                                    argus_daemon::report_memory();
                                } 
                                else if command.contains("open code") {
                                    println!("--> ACTION: Launching IDE...");
                                    argus_daemon::launch_app("Visual Studio Code");
                                } 
                                else if command.contains("open browser") {
                                    println!("--> ACTION: Launching Web...");
                                    argus_daemon::launch_app("Safari"); 
                                } 
                                else if command.contains("clear") && command.contains("cache") {
                                    println!("--> ACTION: Nuke protocol authorized. Clearing bundler cache...");
                                    argus_daemon::clear_bundler_cache();
                                }
                                else if command.contains("nuke") && command.contains("node") {
                                    println!("--> ACTION: Nuke protocol authorized. Rebuilding project...");
                                    argus_daemon::nuke_node_modules();
                                }
                                else if command.contains("sleep") && !command.contains("port") {
                                    println!("--> ACTION: Going dormant...");
                                }

                                // 3. RETURN TO SLEEP
                                is_awake = false;
                                println!("[EYE CLOSED] Task complete.");
                            }
                        } 
                        // 4. TIMEOUT (Waited too long after waking him up)
                        else if is_awake && last_wake_time.elapsed() >= wake_timeout {
                            is_awake = false;
                            println!("\n[EYE CLOSED] Going dormant...");
                        }
                    }
                }
            }
        },
        err_fn,
        None,
    ).expect("Failed to build audio stream.");

    stream.play().expect("Failed to start audio stream.");

    println!("---");
    println!("Argus is listening... (Press Ctrl+C to put him to sleep)");

    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}