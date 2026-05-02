 use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table},
    Terminal,
};
use std::{error::Error, io, process::Command, time::{Duration, Instant}};
use sysinfo::System;

// --- APP STATE ---
struct AppState {
    voice_logs: Vec<String>,
    // CHANGED: Now using Strings so we can dynamically generate them
    active_ports: Vec<(String, String, String)>, 
    
    sys: System, 
    ram_usage: String,
    
    // NEW: A timer to prevent our network scanner from frying your CPU
    last_port_scan: Instant,
}

impl AppState {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();

        Self {
            voice_logs: vec![
                "[System] Core initialized.".to_string(),
                "[System] Network scanner active...".to_string(),
            ],
            active_ports: Vec::new(),
            sys,
            ram_usage: String::new(),
            // Force an immediate scan on boot
            last_port_scan: Instant::now() - Duration::from_secs(10), 
        }
    }

    fn update_telemetry(&mut self) {
        self.sys.refresh_memory();
        let used_gb = self.sys.used_memory() as f64 / 1_073_741_824.0;
        let total_gb = self.sys.total_memory() as f64 / 1_073_741_824.0;
        let percentage = (used_gb / total_gb) * 100.0;
        self.ram_usage = format!("{:.2} GB / {:.2} GB ({:.1}%)", used_gb, total_gb, percentage);
    }

    // NEW: The Upgraded, Bulletproof Network Scanner
    fn update_network(&mut self) {
        if self.last_port_scan.elapsed() < std::time::Duration::from_secs(2) {
            return; 
        }
        self.last_port_scan = std::time::Instant::now();

        let output = std::process::Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN", "-P", "-n"])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut new_ports = Vec::new();

            for line in stdout.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() { continue; }

                let cmd = parts[0].to_string(); 
                let mut extracted_port = String::new();

                // SMARTER PARSING: Hunt for the port number anywhere in the text line
                for part in &parts {
                    if part.contains(':') {
                        // Split by ':' (e.g., localhost:3000 -> 3000)
                        if let Some(potential_port) = part.split(':').last() {
                            // Verify it's actually a valid number
                            if potential_port.parse::<u16>().is_ok() {
                                extracted_port = potential_port.to_string();
                                break;
                            }
                        }
                    }
                }

                if !extracted_port.is_empty() {
                    let friendly_name = match extracted_port.as_str() {
                        "5173" => format!("Vite ({})", cmd),
                        "3000" => format!("React/Next ({})", cmd),
                        "8080" | "8000" => format!("Node API ({})", cmd),
                        "27017" => format!("MongoDB ({})", cmd),
                        "4321" => format!("Astro ({})", cmd), // Added Astro (common for portfolios!)
                        _ => format!("Custom ({})", cmd), // Catch-all for weird framework ports
                    };

                    new_ports.push((friendly_name, extracted_port, "ONLINE".to_string()));
                }
            }

            // Optional: Sort ports numerically so the UI doesn't jump around
            new_ports.sort_by(|a, b| a.1.parse::<u16>().unwrap_or(0).cmp(&b.1.parse::<u16>().unwrap_or(0)));

            if new_ports.is_empty() {
                new_ports.push(("No dev ports found".to_string(), "---".to_string(), "IDLE".to_string()));
            }

            self.active_ports = new_ports;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState::new();
    let res = run_app(&mut terminal, &mut app_state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Argus encountered a fatal error: {:?}", err);
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut AppState) -> io::Result<()> {
    loop {
        // UPDATE ALL LIVE DATA BEFORE DRAWING
        app.update_telemetry();
        app.update_network();

        terminal.draw(|f| {
            let size = f.area();

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(main_chunks[1]);

            // Header
            let header = Paragraph::new(Line::from(vec![
                Span::styled(" ARGUS CLI ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" // v1.0.0 // STATUS: ", Style::default().fg(Color::DarkGray)),
                Span::styled("AWAKE", Style::default().fg(Color::Green)),
            ]))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
            f.render_widget(header, main_chunks[0]);

            // Port Monitor (Now dynamic!)
            let selected_style = Style::default().fg(Color::Cyan);
            let normal_style = Style::default().fg(Color::White);
            
            let header_cells = ["Service", "Port", "Status"]
                .iter()
                .map(|h| Span::styled(*h, Style::default().fg(Color::DarkGray)));
            let header_row = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD)).height(2);
            
            let rows = app.active_ports.iter().map(|item| {
                let status_color = if item.2 == "ONLINE" { Color::Green } else { Color::Yellow };
                Row::new(vec![
                    Span::styled(&item.0, normal_style),
                    Span::styled(&item.1, selected_style),
                    Span::styled(&item.2, Style::default().fg(status_color)),
                ])
            });

            let port_table = Table::new(rows, [Constraint::Percentage(45), Constraint::Percentage(25), Constraint::Percentage(30)])
                .header(header_row)
                .block(Block::default().title(" LIVE NETWORK MAP ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(port_table, body_chunks[0]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(body_chunks[1]);

            // Logs
            let log_text: Vec<Line> = app.voice_logs.iter().map(|log| Line::from(Span::styled(log, normal_style))).collect();
            let logs = Paragraph::new(log_text)
                .block(Block::default().title(" SYSTEM LOGS ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(logs, right_chunks[0]);

            // Telemetry
            let telemetry_text = Paragraph::new(format!("\n > System RAM Allocation: {}", app.ram_usage))
                .style(Style::default().fg(Color::Magenta))
                .block(Block::default().title(" TELEMETRY ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(telemetry_text, right_chunks[1]);
        })?;

        // 50ms loop speed
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(()); 
                }
            }
        }
    }
}