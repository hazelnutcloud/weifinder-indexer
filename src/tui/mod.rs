mod exporter;
mod stats;

use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

pub use exporter::*;
use duckdb::arrow::array::ArrowNativeTypeOp;
use ratatui::{
    Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Widget},
};
pub use stats::*;
use throbber_widgets_tui::{Throbber, ThrobberState, BRAILLE_SIX};
use tokio::task::JoinHandle;

pub struct Tui {
    stats: Arc<Mutex<Stats>>,
    exit: bool,
    fetching_throbber_state: ThrobberState,
    indexing_throbber_state: ThrobberState,
}

impl Tui {
    pub fn spawn(stats: Arc<Mutex<Stats>>) -> JoinHandle<()> {
        let mut terminal = ratatui::init();
        let mut tui = Self {
            stats,
            exit: false,
            fetching_throbber_state: ThrobberState::default(),
            indexing_throbber_state: ThrobberState::default(),
        };

        tokio::task::spawn_blocking(move || {
            while !tui.exit {
                if let Err(_) = terminal.draw(|frame| tui.draw(frame)) {
                    break;
                }
                if let Err(_) = tui.handle_events() {
                    break;
                }
                tui.update();
            }
            ratatui::restore();
        })
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if !event::poll(Duration::from_millis(200))? {
            return Ok(());
        }

        if let Event::Key(key_event) = event::read()?
            && key_event.kind == KeyEventKind::Press
        {
            self.handle_key_event(key_event);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            _ => {}
        }
    }

    fn update(&mut self) {
        self.fetching_throbber_state.calc_next();
        self.indexing_throbber_state.calc_next();
    }
}

impl Widget for &Tui {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let (last_saved_block, last_fetched_block, current_head_number) = {
            let stats = self.stats.lock().unwrap();
            (
                stats
                    .last_saved_block
                    .load(std::sync::atomic::Ordering::Relaxed),
                stats
                    .last_fetched_block
                    .load(std::sync::atomic::Ordering::Relaxed),
                stats
                    .current_head_number
                    .load(std::sync::atomic::Ordering::Relaxed),
            )
        };

        let fetching_progress = (last_fetched_block as f64)
            .div_checked(current_head_number as f64)
            .unwrap_or(0.0);
        let indexing_progress = (last_saved_block as f64)
            .div_checked(current_head_number as f64)
            .unwrap_or(0.0);

        // Create main layout with vertical chunks for the two progress sections
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Fetching progress
                Constraint::Length(3), // Indexing progress
                Constraint::Min(0),    // Remaining space
            ])
            .split(area);

        // Render fetching progress bar with spinner
        render_progress_bar_with_spinner(
            buf,
            main_layout[0],
            "Fetching Progress",
            fetching_progress,
            last_fetched_block,
            current_head_number,
            &self.fetching_throbber_state,
            Color::Cyan,
        );

        // Render indexing progress bar with spinner
        render_progress_bar_with_spinner(
            buf,
            main_layout[1],
            "Indexing Progress",
            indexing_progress,
            last_saved_block,
            current_head_number,
            &self.indexing_throbber_state,
            Color::Green,
        );
    }
}

fn render_progress_bar_with_spinner(
    buf: &mut ratatui::prelude::Buffer,
    area: Rect,
    title: &str,
    progress: f64,
    current: u64,
    total: u64,
    throbber_state: &ThrobberState,
    color: Color,
) {
    // Split the area: spinner (2 chars) | progress bar (rest)
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2), // Spinner
            Constraint::Min(0),    // Progress bar
        ])
        .split(area);

    // Render the throbber/spinner
    let throbber = Throbber::default()
        .throbber_set(BRAILLE_SIX)
        .throbber_style(Style::default().fg(color).add_modifier(Modifier::BOLD));

    // Use the throbber's to_symbol_span to get the current symbol
    let symbol_span = throbber.to_symbol_span(throbber_state);
    buf.set_span(layout[0].x, layout[0].y + 1, &symbol_span, layout[0].width);

    // Render the progress bar
    let label = format!("{}/{} ({:.1}%)", current, total, progress * 100.0);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .gauge_style(Style::default().fg(color).bg(Color::Black))
        .ratio(progress.clamp(0.0, 1.0))
        .label(label);

    gauge.render(layout[1], buf);
}
