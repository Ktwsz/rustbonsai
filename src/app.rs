use std::{io, thread, time};
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
use crate::bonsai::{AsciiChange, BonsaiTree};

pub const TERMINAL_BOUNDS: (u32, u32) = (100, 50);

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

    fn new_point(&mut self, buffer: &Vec<Vec<char>>) {
        let mut vec: Vec<(f64, f64)> = Vec::new();
        for y in 0..buffer[0].len() {
            for x in 0..buffer.len() {
                if buffer[x][y] == '&' {
                    vec.push((x as f64, y as f64));
                }
            }
        }
        self.point.coords = Box::leak(vec.into_boxed_slice());
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
        let bounds: (usize, usize) = (TERMINAL_BOUNDS.0 as usize + 1, TERMINAL_BOUNDS.1 as usize + 1);
        let mut app = App::new();
        let mut terminal = init_terminal()?;
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100);
        let mut buffer = vec![vec![' '; bounds.1]; bounds.0];
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
                app.on_tick(&mut buffer,&mut tree);
                last_tick = Instant::now();
            }
        }

        restore_terminal()

    }
    fn on_tick(&mut self, buffer :&mut Vec<Vec<char>>, tree :&mut BonsaiTree) {
        self.tick_count += 1;
        let ascii_changes = tree.animation_step();

        for change in ascii_changes {
            match change {
                AsciiChange::Change((x, y), c) => buffer[x][y] = c,
                _ => ()
            }
        }
        self.new_point(&buffer);
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