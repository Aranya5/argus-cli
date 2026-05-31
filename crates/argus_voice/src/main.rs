// crates/argus_voice/src/main.rs

mod mappers;
mod router;
mod engine;
mod config;

fn main() {
    engine::ignite();
}