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
use crate::bonsai::{BonsaiTree, PointType};

const TICK_RATE: u64 = 50;

pub struct App<'a> {
    tree_points: Points<'a>,
    tree_marker: Marker,

    leaf_points: Points<'a>,
    leaf_marker: Marker,

    bounds: (f64, f64)
}

impl<'a> App<'a> {
    fn new(terminal_rect: Rect) -> Self {
        Self {
            tree_points: Points {
                coords: &[],
                color: Color::Rgb(205,133,63)
            },
            tree_marker: Marker::Dot,

            leaf_points: Points {
                coords: &[],
                color: Color::Green
            },
            leaf_marker: Marker::Dot,

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
            .marker(self.tree_marker)
            .paint(|ctx| {
                ctx.draw(&self.tree_points);
                ctx.layer();
                ctx.draw(&self.leaf_points);
            })
            .x_bounds([0.0, self.bounds.0])
            .y_bounds([0.0, self.bounds.1])
    }


    pub fn run(seed: Option<u64>, live: bool) -> io::Result<()> {
        let mut terminal = init_terminal()?;

        let terminal_size = terminal.size().unwrap();
        let mut app = App::new(terminal_size);

        let mut tree = BonsaiTree::new(terminal_size, seed);

        tree.generate();
        tree.normalize();

        if !live {
            app.tree_points.coords = Box::leak(tree.get_tree().iter().map(|p| (p.x, p.y)).collect::<Vec<(f64, f64)>>().into_boxed_slice());

            app.leaf_points.coords = Box::leak(tree.get_leaves().iter().map(|p| (p.x, p.y)).collect::<Vec<(f64, f64)>>().into_boxed_slice());
        }

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
            if live && last_tick.elapsed() >= tick_rate {
                app.on_tick(&mut tree);
                last_tick = Instant::now();
            }
        }


        restore_terminal()

    }
    fn on_tick(&mut self, tree: &mut BonsaiTree) {
        let all_changes = tree.animation_step();
        let tree_changes: Vec<(f64, f64)> = all_changes.iter()
            .filter_map(|e| match e {
                PointType::Tree(p) => Some((p.x, p.y)),
                _ => None
            })
            .collect();

        let sliced_changes: &[(f64, f64)] = Box::leak(tree_changes.into_boxed_slice());

        self.tree_points.coords = Box::leak([self.tree_points.coords, sliced_changes].concat().into_boxed_slice());

        let leaves: Vec<(f64, f64)> = all_changes.iter()
            .filter_map(|e| match e {
                PointType::Leaf(p) => Some((p.x, p.y)),
                _ => None
            })
            .collect();

        self.leaf_points.coords = Box::leak([self.leaf_points.coords, Box::leak(leaves.into_boxed_slice())].concat().into_boxed_slice());
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
