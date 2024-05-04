pub mod bonsai;
use bonsai::{BonsaiTree, AsciiChange};
use std::{io, thread, time};
use std::io::{Stdout, stdout};
use crossterm::ExecutableCommand;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Marker, Widget};
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::canvas::{Canvas, Points};
use crate::bonsai::utils::Point;

const TERMINAL_BOUNDS: (u32, u32) = (100, 50);




struct App<'a> {
    x: f64,
    y: f64,
    point: Points<'a>,
    playground: Rect,
    vx: f64,
    vy: f64,
    marker: Marker,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            point: Points {
                coords: &[],
                color: Color::LightCyan,
            },
            playground: Rect::new(10, 10, 200, 100),
            vx: 1.0,
            vy: 1.0,
            marker: Marker::Dot,
        }
    }

    fn new_point(& mut self, buffer: &Vec<Vec<char>>){
        let mut vec: Vec<(f64, f64)> = Vec::new();
        for y in 0..buffer[0].len() {
            for x in 0..buffer.len() {
                if buffer[x][y] == '&' {
                    vec.push((x as f64, y as f64));
                }
            }
        }
        self.point.coords=Box::leak(vec.into_boxed_slice());
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
}

fn main() ->io::Result<()> {
    let mut tree = BonsaiTree::new(TERMINAL_BOUNDS);

    tree.generate();
    tree.normalize();

    let bounds: (usize, usize) = (TERMINAL_BOUNDS.0 as usize + 1, TERMINAL_BOUNDS.1 as usize + 1);
    let mut app = App::new();
    let mut terminal = init_terminal()?;
    let mut buffer = vec![vec![' '; bounds.1]; bounds.0];

    // tree.fill_buffer(&mut buffer);
    // print_buffer(&buffer);
    Ok(for _ in 0..100 {
        let ascii_changes = tree.animation_step();

        for change in ascii_changes {
            match change {
                AsciiChange::Change((x, y), c) => buffer[x][y] = c,
                _ => ()
            }
        }
        app.new_point(&buffer);
        let _ = terminal.draw(|frame| app.ui(frame));

        thread::sleep(time::Duration::from_millis(100));
        restore_terminal()?
    })
}

fn print_buffer(buffer: &Vec<Vec<char>>) {
    for y in 0..buffer[0].len() {
        for x in 0..buffer.len() {
            print!("{}", buffer[x][y]);
        }
        println!("");
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