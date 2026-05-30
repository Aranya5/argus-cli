use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use vosk::{Model, Recognizer};
use std::time::{Instant, Duration};
use crate::router;

fn load_grammar_file(file_path: &str) -> Vec<String> {
    let mut words = Vec::new();
    if let Ok(content) = std::fs::read_to_string(file_path) {
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

pub fn ignite() {
    println!("Awakening Argus...");

    let host = cpal::default_host();
    let mic = host.default_input_device().expect("No microphone detected.");
    let config = mic.default_input_config().expect("Could not read mic config.");
    
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    println!("Argus locked onto: {} ({} Hz, {} Channels)", mic.name().unwrap_or_else(|_| "Unknown".to_string()), sample_rate, channels);

    let model_path = "crates/argus_voice/model";
    let model = Model::new(model_path).expect("CRITICAL FAILURE: Model not found.");

    let dynamic_grammar = load_grammar_file("crates/argus_voice/discovered_grammar.json");
    let allowed_words: Vec<&str> = dynamic_grammar.iter().map(|s| s.as_str()).collect();

    let recognizer = Arc::new(Mutex::new(
        Recognizer::new_with_grammar(&model, sample_rate, &allowed_words).unwrap()
    ));

    println!("Grammar Lock Engaged from external JSON. Background noise will be ignored.");

    let recognizer_clone = recognizer.clone();
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    
    // --- WAKE STATE TRACKERS ---
    let mut is_awake = false;
    let mut last_wake_time = Instant::now();
    let wake_timeout = Duration::from_secs(5);

    // NEW: CUSTOM ENDPOINTING TRACKERS
    let mut last_partial_text = String::new();
    let mut last_partial_time = Instant::now();

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
            let state = rec.accept_waveform(&i16_data);
            
            let mut final_command = String::new();
            let mut trigger_action = false;

            // 1. Check if Vosk naturally finalized (rare in noisy rooms)
            if state == vosk::DecodingState::Finalized {
                if let vosk::CompleteResult::Single(res) = rec.result() {
                    final_command = res.text.to_string();
                    trigger_action = true;
                }
            } else {
                // 2. THE TURBO FIX: Read the partial result stream
                let partial = rec.partial_result().partial.to_string();
                
                if !partial.is_empty() && partial != "[unk]" {
                    if partial == last_partial_text {
                        // If the text hasn't changed in 0.7 seconds, force execution!
                        if last_partial_time.elapsed() > Duration::from_millis(700) {
                            final_command = partial.clone();
                            trigger_action = true;
                            
                            // Wipe Vosk's buffer so it doesn't double-fire
                            rec.reset(); 
                            last_partial_text.clear(); 
                        }
                    } else {
                        // User is still speaking, update trackers
                        last_partial_text = partial;
                        last_partial_time = Instant::now();
                    }
                }
            }

            let command = final_command.trim();

            // 3. EXECUTE THE ROUTING LOGIC
            if trigger_action && !command.is_empty() && command != "[unk]" {
                
                if command.contains("argus") || command.contains("august") {
                    is_awake = true;
                    last_wake_time = Instant::now();
                    println!("\n[EYE OPENED] Yes, Aranya?");
                }

                if is_awake && last_wake_time.elapsed() < wake_timeout {
                    if command != "argus" && command != "august" {
                        
                        // SEND TO ROUTER INSTANTLY
                        router::execute(command);

                        is_awake = false;
                        println!("[EYE CLOSED] Task complete.");
                    }
                } 
                else if is_awake && last_wake_time.elapsed() >= wake_timeout {
                    is_awake = false;
                    println!("\n[EYE CLOSED] Going dormant...");
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