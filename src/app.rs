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

const TICK_RATE: u64 = 100;

pub struct App<'a> {
    point: Points<'a>,
    tick_count: u64,
    marker: Marker,
    bounds: (f64,f64)
}

impl<'a> App<'a> {
    fn new(terminal_rect: Rect) -> Self {
        Self {
            point: Points {
                coords: &[],
                color: Color::Rgb(205,133,63)
            },
            tick_count: 0,
            marker: Marker::Dot,
            bounds: ((terminal_rect.width-terminal_rect.x) as f64,(terminal_rect.height-terminal_rect.y)as f64)
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
            .x_bounds([0.0, self.bounds.0])
            .y_bounds([0.0, self.bounds.1])
    }


    pub fn run(seed: u64, live: bool) -> io::Result<()> {

        let mut terminal = init_terminal()?;
        let mut app = App::new(terminal.size().unwrap());
        let bounds = ((terminal.size().unwrap().width - terminal.size().unwrap().y) as u32,(terminal.size().unwrap().height - terminal.size().unwrap().x)as u32);
        let mut tree = BonsaiTree::new(bounds,seed);
        tree.generate();
        tree.normalize();
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
                if !live{
                    for _ in 0..100{
                        app.on_tick(&mut tree);
                    }}
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
