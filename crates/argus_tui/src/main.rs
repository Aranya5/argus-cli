use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    // 1. SETUP: Take over the terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. RUN: Start the UI loop
    let res = run_app(&mut terminal);

    // 3. TEARDOWN: Give the terminal back to the user
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Argus TUI encountered an error: {:?}", err);
    }

    Ok(())
}

// FIXED: Removed the generic <B: Backend> and explicitly defined Crossterm
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|f| {
            // FIXED: Using area() instead of the deprecated size()
            let size = f.area();

            // Create a main layout block
            let block = Block::default()
                .title(" ARGUS DASHBOARD ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan));

            // Create a simple welcome message
            let paragraph = Paragraph::new("\n\nSystem Online.\n\nPress 'q' to shut down.")
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .block(block);

            // Render the widget to the screen
            f.render_widget(paragraph, size);
        })?; // The ? operator will now work perfectly

        // Listen for keyboard events (like pressing 'q' to quit)
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(()); // Break the loop and exit
                }
            }
        }
    }
}