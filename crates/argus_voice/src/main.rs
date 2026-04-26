use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use vosk::{Model, Recognizer};

// NEW IMPORTS FOR THE LOGGER
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{Read, Write};

// THE AUTO-LOGGER FUNCTION
fn auto_build_grammar(spoken_text: &str) {
    let file_path = "crates/argus_voice/discovered_grammar.json";
    let mut words = HashSet::new();
    
    // 1. Read existing words if the file already exists
    if let Ok(mut file) = std::fs::File::open(file_path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            // Strip out brackets and quotes to read the current list
            let cleaned = content.replace("[", "").replace("]", "").replace("\"", "");
            for word in cleaned.split(',') {
                let w = word.trim();
                if !w.is_empty() && w != "[unk]" {
                    words.insert(w.to_string());
                }
            }
        }
    }

    // 2. Add the new words you just spoke
    let mut changed = false;
    for word in spoken_text.split_whitespace() {
        if words.insert(word.to_string()) {
            changed = true;
        }
    }

    // 3. If it found new words, rewrite the file in Vosk's exact JSON format
    if changed {
        let mut final_list: Vec<String> = words.into_iter().collect();
        final_list.push("[unk]".to_string()); // Always append the unknown flag
        
        let json_string = format!("[\"{}\"]", final_list.join("\", \""));
        
        if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open(file_path) {
            let _ = file.write_all(json_string.as_bytes());
        }
    }
}

fn main() {
    println!("Awakening Argus...");

    // 1. Find the mic and get its EXACT hardware specs
    let host = cpal::default_host();
    let mic = host.default_input_device().expect("No microphone detected.");
    let config = mic.default_input_config().expect("Could not read mic config.");
    
    // We extract the exact sample rate and channel count from your Mac's hardware
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    println!("Argus locked onto: {} ({} Hz, {} Channels)", mic.name().unwrap_or_else(|_| "Unknown".to_string()), sample_rate, channels);

    // 2. Load the Model
    let model_path = "crates/argus_voice/model";
    let model = Model::new(model_path).expect("CRITICAL FAILURE: Model not found.");
    let recognizer = Arc::new(Mutex::new(Recognizer::new(&model, sample_rate).unwrap()));

    println!("Offline brain loaded successfully.");

    let recognizer_clone = recognizer.clone();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    
    // 3. Process the audio stream safely with an Amplifier
    let stream = mic.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| {
            // Software Audio Amplifier
            let gain: f32 = 5.0; // Boosts the raw mic volume by 5x
            
            // We use the `channels` variable here to perfectly slice the stereo audio
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
                    
                    if !command.is_empty() {
                        println!("\nArgus Heard: '{}'", command);
                        
                        // NEW: Dynamically build our future grammar lock
                        auto_build_grammar(command);
                        
                        // THE TRIGGER ENGINE
                        // This is where we match the voice text to actual OS logic
                        match command {
                            "argus kill port three thousand" | "kill port three thousand" => {
                                println!("--> ACTION: Initiating port termination protocol...");
                                // Call the Daemon library!
                                argus_daemon::assassinate_port(3000);
                            },
                            "argus kill port eighty eighty one" | "kill port eighty eighty one" => {
                                println!("--> ACTION: Clearing Metro bundler port...");
                                // Call the Daemon library!
                                argus_daemon::assassinate_port(8081);
                            },
                            "argus sleep" | "sleep" => {
                                println!("--> ACTION: Going dormant...");
                            },
                            _ => {
                                // Silently ignore anything that isn't a command
                            }
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