// crates/argus_voice/src/mappers.rs

pub fn extract_dynamic_port(command: &str) -> Option<u16> {
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
        .replace("oh", "0"); 

    match target.as_str() {
        "eighty eighty" => return Some(8080),
        "eighty eighty one" => return Some(8081),
        "eighty eight" => return Some(88),
        "fifty one seventy three" => return Some(5173),
        "fifty four thirty two" => return Some(5432), 
        "thirty three 0 six" => return Some(3306),    
        "sixty three seventy nine" => return Some(6379), 
        "forty two 0 0" => return Some(4200),         
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

pub fn map_url(spoken: &str) -> Option<&'static str> {
    // We catch the exact words, AND the spaced-out acoustic versions
    match spoken {
        "google" => Some("https://google.com"),
        "github" | "git hub" | "get hub" => Some("https://github.com"),
        "youtube" | "you tube" => Some("https://youtube.com"),
        
        // Developer Localhosts
        "localhost" | "local host" | "local" => Some("http://localhost:3000"), 
        "vite" | "veet" | "light" => Some("http://localhost:5173"), // Vosk sometimes hears "vite" as "veet" or "light"
        "backend" | "back end" => Some("http://localhost:8080"), 
        
        _ => None,
    }
}