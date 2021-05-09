use crate::stats::BambiStats;

use std::{
    io,
    io::{stdout, Stdout},
    sync::Arc,
    thread::JoinHandle,
    time
};
use tui::widgets::Axis;
use tui::widgets::Chart;
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Block, Borders, Dataset},
    Terminal,
};

#[cfg(windows)]
use crossterm::{execute, terminal::enable_raw_mode};
#[cfg(windows)]
use tui::backend::CrosstermBackend;

#[cfg(not(windows))]
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
#[cfg(not(windows))]
use tui::backend::TermionBackend;

pub fn initialize(stats: Arc<BambiStats>) -> Result<(), io::Error> {
    UserInterfaceThread::start(stats);
    Ok(())
}

#[cfg(windows)]
struct UserInterfaceThread {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

#[cfg(not(windows))]
struct UserInterfaceThread {
    terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<Stdout>>>>,
    stats: Arc<BambiStats>,
    ok_flags_data: Vec<(time::Duration, u64)>,
}

const FLAGS_DATAPOINT_COUNT: isize = 120;

impl UserInterfaceThread {
    #[cfg(target_os = "windows")]
    fn start() -> JoinHandle<()> {
        //enable_raw_mode().unwrap();

        std::thread::spawn(|| {
            let stdout = io::stdout();
            let backend = CrosstermBackend::new(stdout);
            let terminal = Terminal::new(backend).expect("Terminal init failed");
            UserInterfaceThread { terminal: terminal }.run();
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn start(stats: Arc<BambiStats>) -> JoinHandle<()> {
        //enable_raw_mode().unwrap();

        std::thread::spawn(|| {
            let stdout = io::stdout();
            let stdout = MouseTerminal::from(stdout);
            let stdout = AlternateScreen::from(stdout);
            let backend = TermionBackend::new(stdout);
            let terminal = Terminal::new(backend).expect("Terminal init failed");

            UserInterfaceThread {
                terminal: terminal,
                stats: stats,
                ok_flags_data: vec![]
            }
            .run();
        })
    }

    fn run(&mut self) {
        loop {
            self.update_frame().unwrap();
        }
    }

    fn update_frame(&mut self) -> io::Result<()> {
        //let t = (0..100).map(|t| {let t = t as f64; t.sin_cos()}).collect::<Vec<_>>();

        let start = 0.max(self.ok_flags_data.len() as isize - FLAGS_DATAPOINT_COUNT) as usize;
        let ok_flags: Vec<(f64, f64)> = self.ok_flags_data[start..].into_iter().map(|(timestamp, ok_count)| (timestamp.as_secs_f64(), *ok_count as f64)).collect();
        
        let start_time = self.ok_flags_data[start].0;
        self.terminal.draw(|f| {
            //let t = 0..30+

            let size = f.size();

            let container = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let ok_flags_dataset = Dataset::default()
                .name("ok_flags")
                .marker(symbols::Marker::Braille)
                .data(&ok_flags);

            let submission_chart = Chart::new(vec![ok_flags_dataset])
                .block(Block::default().borders(Borders::ALL))
                .x_axis(Axis::default().bounds([0.0, 12.0]))
                .y_axis(
                    Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels(vec![
                            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                            Span::styled("12", Style::default().add_modifier(Modifier::BOLD)),
                        ])
                        .bounds([0.0, 12.0]),
                );

            f.render_widget(submission_chart, container[0]);
        })?;
        Ok(())
    }
}
