use crate::{
    events::{Event, EVENTS, Events},
    stats::BambiStats,
};

use std::{
    io,
    io::{stdout, Stdout},
    sync::Arc,
    thread::JoinHandle,
    time::{self, Duration, Instant},
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
use termion::{
    event::Key,
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
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
    terminal: Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>,
    stats: Arc<BambiStats>,
    ok_flags_data: Vec<(Instant, f64)>,
}

const FLAGS_DATAPOINT_COUNT: isize = 120;
const N_YTICKS: usize = 5;

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
            {
                
            }
            let stdout = io::stdout()
                .into_raw_mode()
                .expect("Failed to set Terminal into raw mode");
            let stdout = MouseTerminal::from(stdout);
            let stdout = AlternateScreen::from(stdout);
            let backend = TermionBackend::new(stdout);
            let terminal = Terminal::new(backend).expect("Terminal init failed");

            {
                UserInterfaceThread {
                    terminal: terminal,
                    stats: stats,
                    ok_flags_data: vec![],
                }
                .run();
            }

            std::process::exit(0);
        })
    }

    fn run(&mut self) {
        let mut events = &mut *EVENTS.lock().unwrap();
        for event in events {
            match event {
                Event::Tick => self.update_frame().unwrap(),
                Event::Input(key) => {
                    if key == Key::Ctrl('c') {
                        return
                    }
                },
            }
        }
    }

    fn update_frame(&mut self) -> io::Result<()> {
        let start_time = time::Instant::now();
        let interval = Duration::from_secs(15);

        // self.ok_flags_data = (0..1000)
        //     .map(|t| (start_time + t * interval, (t as f64 / 20.0).sin()))
        //     .collect::<Vec<_>>();

        self.ok_flags_data.push((Instant::now(), self.stats.get_ok() as _));

        let start = 0.max(self.ok_flags_data.len() as isize - FLAGS_DATAPOINT_COUNT) as usize;
        let start_time = self.ok_flags_data[start].0;

        let ok_flags: Vec<(f64, f64)> = self.ok_flags_data[start..]
            .into_iter()
            .map(|(timestamp, ok_count)| {
                ((*timestamp - start_time).as_secs_f64(), *ok_count as f64)
            })
            .collect();

        let x_bounds = [ok_flags.first().unwrap().0, ok_flags.last().unwrap().0];
        let y_bounds: [f64; 2] = ok_flags.iter().fold([f64::INFINITY, f64::NEG_INFINITY], |bounds, value| {
            [bounds[0].min(value.1), bounds[1].max(value.1)]
        });
        let y_bounds = [y_bounds[0].floor(), y_bounds[1].ceil()];
        let y_interval = (y_bounds[1] - y_bounds[0]) / (N_YTICKS as f64 - 1.0);
        //println!("{}", y_interval);
        let y_ticks: Vec<_> = (0..N_YTICKS)
            .map(|t| y_bounds[0] + y_interval * (t as f64))
            .map(|y_tick| {
                Span::styled(
                    format!("{:#.4}", y_tick),
                    Style::default().add_modifier(Modifier::BOLD),
                )
            })
            .collect();
        //println!("{:?}", y_ticks);
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
                .x_axis(Axis::default().bounds(x_bounds))
                .y_axis(
                    Axis::default()
                        .title("Accepted Flags")
                        .style(Style::default().fg(Color::Gray))
                        .labels(y_ticks)
                        .bounds(y_bounds),
                );

            f.render_widget(submission_chart, container[0]);
        })?;
        Ok(())
    }
}
