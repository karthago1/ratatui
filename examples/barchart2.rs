use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{BarChart2, Block, Borders},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

struct App<'a> {
    data: Vec<u64>,
    data2: Vec<u64>,
    labels: Vec<&'a str>,
    styles: Vec<Style>,
    value_styles: Vec<Style>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            data: vec![9, 12, 5, 8],
            data2: vec![6, 11, 4, 5],
            labels: vec!["30°C", "50°C", "60°C", "80°C"],
            styles: vec![
                Style::default().fg(Color::Green),
                Style::default().fg(Color::Yellow),
            ],
            value_styles: vec![
                Style::default().bg(Color::Green).fg(Color::Black),
                Style::default().bg(Color::Yellow).fg(Color::Black),
            ],
        }
    }

    fn on_tick(&mut self) {}
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let barchart = BarChart2::default()
        .block(Block::default().title("Data1").borders(Borders::ALL))
        .add_data(&app.data)
        .add_data(&app.data2)
        .add_data(&app.data2)
        .bar_width(9)
        .bar_styles(&app.styles)
        .labels(&app.labels)
        .value_format(|v| (v + 20).to_string())
        .value_styles(&app.value_styles);
    f.render_widget(barchart, chunks[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let barchart = BarChart2::default()
        .block(Block::default().title("Data2").borders(Borders::ALL))
        .add_data(&app.data)
        .add_data(&app.data2)
        .bar_width(5)
        .group_gap(3)
        .bar_styles(&app.styles)
        .value_styles(&app.value_styles);

    f.render_widget(barchart, chunks[0]);

    /*let barchart = BarChart2::default()
        .block(Block::default().title("Data3").borders(Borders::ALL))
        .add_data(&app.data)
        .add_data(&app.data2)
        .bar_style(Style::default().fg(Color::Red))
        .bar_width(7)
        .bar_gap(0)
        .value_style(Style::default().bg(Color::Red))
        .label_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),
        );
    f.render_widget(barchart, chunks[1]);*/
}
