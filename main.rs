use std::error::Error;
use std::io;

mod probe_data;
mod optimizer;

#[cfg(feature = "api")]
mod api;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
    Terminal,
};

#[cfg(not(feature = "api"))]
fn main() -> Result<(), Box<dyn Error>> {
    // TUI mode when api feature is not enabled
    let file_path = "data.csv";
    let all_data = probe_data::read_csv(file_path)?;
    let recent_data = probe_data::filter_last_n_days(&all_data, 3);
    let averages = probe_data::calculate_averages(&recent_data);

    // Setup terminal UI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the user interface
    let result = run_ui(&mut terminal, averages);

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result?;
    Ok(())
}

#[cfg(feature = "api")]
fn main() -> Result<(), Box<dyn Error>> {
    // API mode - use actix's runtime instead of tokio directly
    let optimizer = std::sync::Arc::new(std::sync::Mutex::new(optimizer::NetworkOptimizer::new()));
    
    actix_web::rt::System::new().block_on(async {
        api::run(optimizer).await
    })?;
    
    Ok(())
}

fn run_ui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    averages: probe_data::Averages,
) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .split(frame.size());

            let latency_value = display_opt(averages.latency);
            let jitter_value = display_opt(averages.jitter);
            let packet_loss_value = display_opt(averages.packet_loss);
            let signal_strength_value = display_opt(averages.signal_strength);
            let download_speed_value = display_opt(averages.download_speed);
            let upload_speed_value = display_opt(averages.upload_speed);

            let rows = vec![
                Row::new(vec!["Latency (ms)", &latency_value]),
                Row::new(vec!["Jitter (ms)", &jitter_value]),
                Row::new(vec!["Packet Loss (%)", &packet_loss_value]),
                Row::new(vec!["Signal Strength (%)", &signal_strength_value]),
                Row::new(vec!["Download Speed (Mbps)", &download_speed_value]),
                Row::new(vec!["Upload Speed (Mbps)", &upload_speed_value]),
            ];

            let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Percentage(50)])
                .block(Block::default().borders(Borders::ALL).title("Network Averages (Last 3 Days)"))
                .column_spacing(2)
                .style(Style::default().fg(Color::White));

            frame.render_widget(table, layout[0]);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn display_opt(opt: Option<f64>) -> String {
    match opt {
        Some(value) => format!("{:.2}", value),
        None => "N/A".to_string(),
    }
}
