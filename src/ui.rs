use crate::{
    events::{Event, Events, EVENTS},
    stats::BambiStats,
};

use std::{
    io,
    io::{stdout, Stdout},
    sync::Arc,
    thread::JoinHandle,
    time::{self, Duration, Instant},
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, Paragraph, Row, Table, Tabs, Wrap},
    Frame, Terminal,
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
    uistate: UiState,
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
                    uistate: UiState {
                        close_menu: false,
                        tab_selected: TabState::Overview,
                        overview_data: UiOverviewState {
                            ok_flags_data: vec![],
                        },
                        team_state: TeamState { teams_data: vec![] },
                    },
                }
                .run();
            }

            std::process::exit(0);
        })
    }

    fn run(&mut self) {
        let events = &mut *EVENTS.lock().unwrap();
        for event in events {
            match event {
                Event::Tick => self.update_frame().unwrap(),
                Event::Input(key) => {
                    if key == Key::Ctrl('c') {
                        if self.uistate.close_menu {
                            return;
                        }
                        self.uistate.close_menu = true;
                    } else {
                        self.uistate.close_menu = false;
                    }

                    if key == Key::Left {
                        self.uistate.tab_selected.previous();
                    }

                    if key == Key::Right {
                        self.uistate.tab_selected.next();
                    }
                }
            }
        }
    }

    fn update_frame(&mut self) -> io::Result<()> {
        // Record a new data point
        self.uistate
            .overview_data
            .ok_flags_data
            .push((Instant::now(), self.stats.get_ok() as _));

        let uistate = &self.uistate;
        let _ = self.terminal.draw(|f| uistate.draw_ui(f));
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum TabState {
    Overview = 0,
    Teams,
    Exploits,
}

impl TabState {
    fn next(&mut self) {
        *self = match self {
            Self::Overview => Self::Teams,
            Self::Teams => Self::Exploits,
            Self::Exploits => Self::Exploits,
        }
    }

    fn previous(&mut self) {
        *self = match self {
            Self::Overview => Self::Overview,
            Self::Teams => Self::Overview,
            Self::Exploits => Self::Teams,
        }
    }

    fn get_titles() -> Vec<&'static str> {
        return vec!["Overview", "Teams", "Exploits"];
    }

    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let titles = Self::get_titles()
            .into_iter()
            .map(|title| Spans::from(Span::styled(title, Style::default().fg(Color::Green))))
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("BambiXploit"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(*self as _);
        f.render_widget(tabs, area);
    }
}

struct UiOverviewState {
    ok_flags_data: Vec<(Instant, f64)>,
}

struct FlagStats {
    ok_flags: usize,
    invalid_flags: usize,
    last_valid_flag: Instant,
    // history
}

struct TeamData {
    enabled: bool,
    address: String,
    team_stats: FlagStats,
}

struct TeamState {
    teams_data: Vec<TeamData>,
}

struct UiState {
    tab_selected: TabState,
    close_menu: bool,
    overview_data: UiOverviewState,
    team_state: TeamState,
}

impl UiState {
    fn draw_ui<B: Backend>(&self, f: &mut Frame<B>) {
        let base_rect = f.size();
        let base_chunks = Layout::default()
            .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
            .split(base_rect);

        self.tab_selected.draw(f, base_chunks[0]);

        match self.tab_selected {
            TabState::Overview => self.draw_overview(f, base_chunks[1]),
            TabState::Teams => {}
            TabState::Exploits => {}
        }

        if self.close_menu {
            let popup_area = centered_rect(20, 40, base_rect);
            let popup_text = vec![
                Spans::from(vec![
                    Span::raw("Really quit? ("),
                    Span::styled("^C", Style::default().fg(Color::Red)),
                    Span::raw(")"),
                ]),
                Spans::from(vec![Span::raw(
                    "All running exploits will not be able to submit their Flags!",
                )]),
            ];

            let popup_text = Paragraph::new(popup_text)
                .block(
                    Block::default()
                        .title("Quit?")
                        .borders(Borders::TOP | Borders::BOTTOM),
                )
                .style(Style::default())
                .wrap(Wrap { trim: true });
            f.render_widget(popup_text, popup_area);
        }
    }

    fn draw_overview<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Select the last n data points
        let start =
            0.max(self.overview_data.ok_flags_data.len() as isize - FLAGS_DATAPOINT_COUNT) as usize;
        let start_time = self.overview_data.ok_flags_data[start].0;
        let ok_flags: Vec<(f64, f64)> = self.overview_data.ok_flags_data[start..]
            .into_iter()
            .map(|(timestamp, ok_count)| ((*timestamp - start_time).as_secs_f64(), *ok_count))
            .collect();

        // Calculate the bounds of the axes
        let x_bounds = [ok_flags.first().unwrap().0, ok_flags.last().unwrap().0];
        let y_bounds: [f64; 2] = ok_flags
            .iter()
            .fold([f64::INFINITY, f64::NEG_INFINITY], |bounds, value| {
                [bounds[0].min(value.1), bounds[1].max(value.1)]
            });
        let y_bounds = [y_bounds[0].floor(), y_bounds[1].ceil()];
        let y_bounds = [y_bounds[0], y_bounds[1].max(y_bounds[0] + 1.0)]; // the y axis should span at least one flag
        let y_interval = (y_bounds[1] - y_bounds[0]) / (N_YTICKS as f64 - 1.0);

        let y_ticks: Vec<_> = (0..N_YTICKS)
            .map(|t| y_bounds[0] + y_interval * (t as f64))
            .map(|y_tick| {
                Span::styled(
                    format!("{:#5.2}", y_tick),
                    Style::default().add_modifier(Modifier::BOLD),
                )
            })
            .collect();

        let container = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area);

        let ok_flags_dataset = Dataset::default()
            //.name("ok_flags")
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
    }

    fn draw_team_menu<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let team_table: Vec<_> = self
            .team_state
            .teams_data
            .iter()
            .enumerate()
            .map(|(id, team)| {
                let enabled = if team.enabled {
                    Span::raw("✔️")
                } else {
                    Span::raw("❌")
                };

                Row::new(vec![
                    Cell::from(id.to_string()),
                    Cell::from(enabled),
                    Cell::from(team.address.as_str()),
                ])
            })
            .collect();
        Table::new(team_table);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
