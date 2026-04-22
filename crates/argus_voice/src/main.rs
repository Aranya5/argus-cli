use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use vosk::{Model, Recognizer};

fn main() {
    println!("Awakening Argus...");

    // 1. Find the mic and get its EXACT hardware specs
    let host = cpal::default_host();
    let mic = host.default_input_device().expect("No microphone detected.");
    let config = mic.default_input_config().expect("Could not read mic config.");
    
    // --> THESE ARE THE MISSING LINES <--
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
                    // Prevent it from printing empty blank spaces
                    if !res.text.trim().is_empty() {
                        println!("Argus Heard: {}", res.text);
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