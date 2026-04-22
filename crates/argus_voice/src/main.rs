use cpal::traits::{DeviceTrait, HostTrait};
use vosk::{Model, Recognizer};

fn main() {
    println!("Awakening Argus...");

    // 1. Load the Offline Language Model
    // Since we run this from the root workspace, we point to the crate's folder.
    let model_path = "crates/argus_voice/model";
    
    let model = Model::new(model_path)
        .expect("CRITICAL FAILURE: Could not find the offline model. Is the folder extracted and named correctly?");
    
    // Set the sample rate to 16kHz (standard for Vosk voice models)
    let _recognizer = Recognizer::new(&model, 16000.0)
        .expect("CRITICAL FAILURE: Could not initialize the voice recognizer.");

    println!("Offline brain loaded successfully.");

    // 2. Tap into the Operating System's Audio Hardware
    let host = cpal::default_host();
    
    // Find the default microphone
    let mic = host.default_input_device()
        .expect("CRITICAL FAILURE: No microphone detected on this system.");
    
    // Fetch the OS-level configuration for the mic
    let config = mic.default_input_config()
        .expect("CRITICAL FAILURE: Could not read microphone configuration.");

    println!("Argus is locked onto audio device: {}", mic.name().unwrap_or_else(|_| "Unknown Mic".to_string()));
    println!("Audio Format: {:?}", config.sample_format());
    
    println!("---");
    println!("System Check Complete. Argus is ready for the audio stream.");
}