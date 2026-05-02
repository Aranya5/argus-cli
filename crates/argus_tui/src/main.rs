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
use std::{error::Error, io};
use sysinfo::System; // NEW: The hardware monitor

// --- APP STATE ---
struct AppState {
    voice_logs: Vec<String>,
    active_ports: Vec<(&'static str, u16, &'static str)>,
    
    // NEW: Live system monitoring variables
    sys: System, 
    ram_usage: String,
}

impl AppState {
    fn new() -> Self {
        // Initialize the hardware monitor
        let mut sys = System::new_all();
        sys.refresh_memory();

        Self {
            voice_logs: vec![
                "[10:42 AM] System Online. Awaiting voice input...".to_string(),
                "[10:45 AM] HEARD: 'open site github'".to_string(),
                "[10:45 AM] ACTION: Launching browser...".to_string(),
            ],
            active_ports: vec![
                ("Vite/React", 5173, "ONLINE"),
                ("Node/Express", 8080, "ONLINE"),
                ("MongoDB", 27017, "LISTENING"),
                ("Rust Daemon", 9999, "IDLE"),
            ],
            sys,
            ram_usage: String::new(),
        }
    }

    // NEW: Function to recalculate RAM every frame
    fn update_telemetry(&mut self) {
        self.sys.refresh_memory();
        
        // Convert bytes to Gigabytes
        let used_gb = self.sys.used_memory() as f64 / 1_073_741_824.0;
        let total_gb = self.sys.total_memory() as f64 / 1_073_741_824.0;
        let percentage = (used_gb / total_gb) * 100.0;

        self.ram_usage = format!("{:.2} GB / {:.2} GB ({:.1}%)", used_gb, total_gb, percentage);
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
        // 1. UPDATE DATA BEFORE DRAWING
        app.update_telemetry();

        // 2. DRAW THE UI
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

            // Port Monitor (Sidebar)
            let selected_style = Style::default().fg(Color::Cyan);
            let normal_style = Style::default().fg(Color::White);
            
            let header_cells = ["Service", "Port", "Status"]
                .iter()
                .map(|h| Span::styled(*h, Style::default().fg(Color::DarkGray)));
            let header_row = Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD)).height(2);
            
            let rows = app.active_ports.iter().map(|item| {
                let status_color = if item.2 == "ONLINE" { Color::Green } else { Color::Yellow };
                Row::new(vec![
                    Span::styled(item.0, normal_style),
                    Span::styled(item.1.to_string(), selected_style),
                    Span::styled(item.2, Style::default().fg(status_color)),
                ])
            });

            let port_table = Table::new(rows, [Constraint::Percentage(40), Constraint::Percentage(30), Constraint::Percentage(30)])
                .header(header_row)
                .block(Block::default().title(" NETWORK MANAGER ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(port_table, body_chunks[0]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(body_chunks[1]);

            // Logs
            let log_text: Vec<Line> = app.voice_logs.iter().map(|log| Line::from(Span::styled(log, normal_style))).collect();
            let logs = Paragraph::new(log_text)
                .block(Block::default().title(" VOICE PROTOCOL LOGS ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(logs, right_chunks[0]);

            // Telemetry (Now using live data!)
            let telemetry_text = Paragraph::new(format!("\n > System RAM Allocation: {}", app.ram_usage))
                .style(Style::default().fg(Color::Magenta))
                .block(Block::default().title(" TELEMETRY ").borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            f.render_widget(telemetry_text, right_chunks[1]);
        })?;

        // 3. LISTEN FOR KEYBOARD INPUT
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(()); 
                }
            }
        }
    }
}