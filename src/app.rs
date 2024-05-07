use std::io;
use std::io::{Stdout, stdout};
use std::time::{Duration, Instant};
use crossterm::{event, ExecutableCommand};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Marker, Widget};
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::canvas::{Canvas, Points};
use crate::bonsai::BonsaiTree;

pub const TERMINAL_BOUNDS: (u32, u32) = (100, 50);

const TICK_RATE: u64 = 100;

pub struct App<'a> {
    point: Points<'a>,
    tick_count: u64,
    marker: Marker,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            point: Points {
                coords: &[],
                color: Color::LightCyan,
            },
            tick_count: 0,
            marker: Marker::Dot,
        }
    }

    fn ui(&self, frame: &mut Frame) {
        let tree = Rect::new(0, 0, frame.size().width, frame.size().height);
        frame.render_widget(self.tree_canvas(), tree);
    }
    fn tree_canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Bonsai"))
            .marker(self.marker)
            .paint(|ctx| {
                ctx.draw(&self.point);
            })
            .x_bounds([10.0, 210.0])
            .y_bounds([10.0, 110.0])
    }


    pub fn run(mut tree: BonsaiTree) -> io::Result<()> {
        let mut app = App::new();
        let mut terminal = init_terminal()?;
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(TICK_RATE);

        loop {
            let _ = terminal.draw(|frame| app.ui(frame));
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                app.on_tick(&mut tree);
                last_tick = Instant::now();
            }
        }

        restore_terminal()

    }
    fn on_tick(&mut self, tree: &mut BonsaiTree) {
        self.tick_count += 1;
        let changes: Vec<(f64, f64)> = tree.animation_step()
            .iter()
            .map(|p| (p.x, p.y))
            .collect();

        let sliced_changes: &[(f64, f64)] = Box::leak(changes.into_boxed_slice());

        self.point.coords = Box::leak([self.point.coords, sliced_changes].concat().into_boxed_slice());
    }
}
fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}
fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
