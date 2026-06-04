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
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table, Tabs},
    Terminal,
};
use std::{error::Error, io::{self, BufRead}, time::{Duration, Instant}};
use sysinfo::System;

// --- APP STATE ---
struct AppState {
    voice_logs: Vec<String>,
    active_ports: Vec<(String, String, String)>, 
    
    // Store Docker data (Name, Status, Ports)
    docker_containers: Vec<(String, String, String)>,
    // Timer to prevent spamming the Docker daemon
    last_docker_scan: Instant, 
    
    sys: System, 
    ram_usage: String,
    last_port_scan: Instant,
    active_tab: usize, 
}

impl AppState {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();

        Self {
            voice_logs: vec!["[System] Core initialized.".to_string()],
            active_ports: Vec::new(),
            
            docker_containers: Vec::new(),
            last_docker_scan: Instant::now() - Duration::from_secs(10),
            
            sys,
            ram_usage: String::new(),
            last_port_scan: Instant::now() - Duration::from_secs(10),
            active_tab: 0, 
        }
    }

    fn update_telemetry(&mut self) {
        self.sys.refresh_memory();
        // Assuming newer sysinfo (Bytes to GB)
        let used_gb = self.sys.used_memory() as f64 / 1_073_741_824.0;
        let total_gb = self.sys.total_memory() as f64 / 1_073_741_824.0;
        let percentage = (used_gb / total_gb) * 100.0;
        self.ram_usage = format!("{:.2} GB / {:.2} GB ({:.1}%)", used_gb, total_gb, percentage);
    }

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
                let mut port_number: u16 = 0;

                for part in &parts {
                    if part.contains(':') {
                        if let Some(potential_port) = part.split(':').last() {
                            if let Ok(num) = potential_port.parse::<u16>() {
                                extracted_port = potential_port.to_string();
                                port_number = num;
                                break;
                            }
                        }
                    }
                }

                let is_dev_port = match port_number {
                    3000..=3010 => true, 
                    4000..=4010 => true, 
                    4200 => true,        
                    4321 => true,        
                    5000..=5001 => true, 
                    5173 => true,        
                    8000..=8080 => true, 
                    27017 => true,       
                    _ => false,          
                };

                if is_dev_port && !extracted_port.is_empty() {
                    let friendly_name = match port_number {
                        5173 => format!("Vite ({})", cmd),
                        3000 => format!("React/Next ({})", cmd),
                        8000..=8080 => format!("Node API ({})", cmd),
                        27017 => format!("MongoDB ({})", cmd),
                        4321 => format!("Astro ({})", cmd),
                        _ => format!("Dev Port ({})", cmd),
                    };

                    new_ports.push((friendly_name, extracted_port, "ACTIVE".to_string()));
                }
            }

            new_ports.sort_by(|a, b| a.1.parse::<u16>().unwrap_or(0).cmp(&b.1.parse::<u16>().unwrap_or(0)));

            if new_ports.is_empty() {
                new_ports.push(("System Clear".to_string(), "---".to_string(), "IDLE".to_string()));
            }

            self.active_ports = new_ports;
        }
    }
    
    fn update_docker(&mut self) {
        if self.last_docker_scan.elapsed() < std::time::Duration::from_secs(2) {
            return; 
        }
        self.last_docker_scan = std::time::Instant::now();

        let output = std::process::Command::new("docker")
            .args(["ps", "--format", "{{.Names}}|{{.Status}}|{{.Ports}}"])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut containers = Vec::new();

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 3 {
                    let name = parts[0].to_string();
                    let status = parts[1].to_string();
                    
                    let raw_ports = parts[2].to_string();
                    let clean_ports = if raw_ports.is_empty() { 
                        "No mapped ports".to_string() 
                    } else { 
                        raw_ports.replace("0.0.0.0:", "").replace("->", " => ") 
                    };

                    containers.push((name, status, clean_ports));
                }
            }

            if containers.is_empty() {
                containers.push(("System Clear".to_string(), "NO CONTAINERS RUNNING".to_string(), "---".to_string()));
            }

            self.docker_containers = containers;
        }
    }

    fn update_logs(&mut self) {
        if let Ok(file) = std::fs::File::open("/tmp/argus.log") {
            let reader = std::io::BufReader::new(file);
            let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
            
            // Allow up to 50 logs in history for dynamic slicing
            let tail: Vec<String> = lines.into_iter().rev().take(50).rev().collect();
            
            if !tail.is_empty() {
                self.voice_logs = tail;
            }
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
        // UPDATE ALL LIVE DATA
        app.update_telemetry();
        app.update_network();
        app.update_logs(); 
        app.update_docker(); // ADDED: Docker scanner now runs

        terminal.draw(|f| {
            let size = f.area();

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), 
                    Constraint::Length(3), 
                    Constraint::Min(0)     
                ].as_ref())
                .split(size);

            let header = Paragraph::new(Line::from(vec![
                Span::styled(" ARGUS CLI ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" // v1.1.0 // STATUS: ", Style::default().fg(Color::DarkGray)),
                Span::styled("AWAKE", Style::default().fg(Color::Green)),
            ]))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
            f.render_widget(header, main_chunks[0]);

            let tab_titles = vec![" [1] NETWORK ", " [2] DOCKER ", " [3] SERVER LOGS "];
            let tabs = Tabs::new(tab_titles)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
                .select(app.active_tab);
            f.render_widget(tabs, main_chunks[1]);

            match app.active_tab {
                0 => {
                    // --- TAB 0: NETWORK SCREEN ---
                    let body_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                        .split(main_chunks[2]); 

                    let selected_style = Style::default().fg(Color::Cyan);
                    let normal_style = Style::default().fg(Color::White);
                    let header_cells = ["Service", "Port", "Status"].iter().map(|h| Span::styled(*h, Style::default().fg(Color::DarkGray)));
                    let header_row = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD)).height(2);
                    
                    let rows = app.active_ports.iter().map(|item| {
                        let status_color = if item.2 == "ACTIVE" { Color::Green } else { Color::DarkGray };
                        Row::new(vec![
                            Span::styled(&item.0, normal_style),
                            Span::styled(&item.1, selected_style),
                            Span::styled(&item.2, Style::default().fg(status_color)),
                        ])
                    });

                    let port_table = Table::new(rows, [Constraint::Percentage(45), Constraint::Percentage(25), Constraint::Percentage(30)])
                        .header(header_row)
                        .block(Block::default().title(" LIVE NETWORK MAP ").borders(Borders::ALL).border_type(BorderType::Rounded));
                    f.render_widget(port_table, body_chunks[0]);

                    let right_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                        .split(body_chunks[1]);

                    let max_visible_lines = right_chunks[0].height.saturating_sub(2) as usize;
                    let visible_logs: Vec<&String> = app.voice_logs.iter().rev().take(max_visible_lines).rev().collect();
                    let log_text: Vec<Line> = visible_logs.iter().map(|log| Line::from(Span::styled(*log, normal_style))).collect();
                    let logs = Paragraph::new(log_text).block(Block::default().title(" SYSTEM LOGS ").borders(Borders::ALL));
                    f.render_widget(logs, right_chunks[0]);

                    let telemetry_text = Paragraph::new(format!("\n > System RAM Allocation: {}", app.ram_usage))
                        .style(Style::default().fg(Color::Magenta))
                        .block(Block::default().title(" TELEMETRY ").borders(Borders::ALL));
                    f.render_widget(telemetry_text, right_chunks[1]);
                }
                1 => {
                    // --- TAB 1: DOCKER VISUALIZER ---
                    let normal_style = Style::default().fg(Color::White);
                    let container_style = Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD);
                    
                    let header_cells = ["Container Name", "Status", "Port Mappings"]
                        .iter()
                        .map(|h| Span::styled(*h, Style::default().fg(Color::DarkGray)));
                    
                    let header_row = Row::new(header_cells)
                        .style(Style::default().add_modifier(Modifier::BOLD))
                        .height(2);
                    
                    let rows = app.docker_containers.iter().map(|item| {
                        // Color code the status: Green for "Up", Red for anything else
                        let status_color = if item.1.starts_with("Up") { Color::Green } else { Color::Red };
                        
                        Row::new(vec![
                            Span::styled(&item.0, container_style),
                            Span::styled(&item.1, Style::default().fg(status_color)),
                            Span::styled(&item.2, normal_style),
                        ])
                    });

                    let docker_table = Table::new(rows, [Constraint::Percentage(25), Constraint::Percentage(30), Constraint::Percentage(45)])
                        .header(header_row)
                        .block(Block::default()
                            .title(" DOCKER CONTAINERS ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Blue)));
                    
                    f.render_widget(docker_table, main_chunks[2]);
                }
                2 => {
                    // --- TAB 2: SERVER LOG TAILING ---
                    let logs_placeholder = Paragraph::new("\n\n  [ Listening for localhost output... ]\n  No active server crashes detected.")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Red))
                        .block(Block::default().title(" LIVE SERVER LOGS ").borders(Borders::ALL));
                    f.render_widget(logs_placeholder, main_chunks[2]);
                }
                _ => {}
            }
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right => app.active_tab = (app.active_tab + 1) % 3,
                    KeyCode::Left => app.active_tab = (app.active_tab + 2) % 3,
                    KeyCode::Char('1') => app.active_tab = 0,
                    KeyCode::Char('2') => app.active_tab = 1,
                    KeyCode::Char('3') => app.active_tab = 2,
                    _ => {}
                }
            }
        }
    }
}