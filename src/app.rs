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

    leaf_points: Points<'a>,

    pot_points: Points<'a>,

    particles: Points <'a>,

    marker: Marker,

    bounds: (f64, f64)
}

impl<'a> App<'a> {
    fn new(terminal_rect: Rect, cherry: bool) -> Self {
        Self {
            tree_points: Points {
                coords: &[],
                color: if cherry { Color::Rgb(205,133,63) } else { Color::Rgb(205,133,63) }
            },

            leaf_points: Points {
                coords: &[],
                color: if cherry { Color::Green } else { Color::Green }
            },

            pot_points: Points {
                coords: &[],
                color: if cherry { Color::Magenta } else { Color::Magenta }
            },

            particles: Points {
                coords: &[],
                color: if cherry { Color::Red } else { Color::Red }
            },

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
                ctx.draw(&self.pot_points);
                ctx.layer();
                ctx.draw(&self.tree_points);
                ctx.layer();
                ctx.draw(&self.leaf_points);
                ctx.layer();
                ctx.draw(&self.particles);
            })
            .x_bounds([0.0, self.bounds.0])
            .y_bounds([0.0, self.bounds.1])
    }


    pub fn run(seed: Option<u64>, live: bool, cherry: bool) -> io::Result<()> {
        let mut terminal = init_terminal()?;

        let terminal_size = terminal.size().unwrap();
        let mut app = App::new(terminal_size, cherry);

        let mut tree = BonsaiTree::new(terminal_size, seed);

        tree.generate();
        tree.normalize();

        app.pot_points.coords = Box::leak(tree.get_pot().into_boxed_slice());

        if !live {
            app.tree_points.coords = Box::leak(tree.get_tree().into_boxed_slice());

            app.leaf_points.coords = Box::leak(tree.get_leaves().into_boxed_slice());
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
            if last_tick.elapsed() >= tick_rate {
                app.on_tick(&mut tree, live);
                last_tick = Instant::now();
            }
        }


        restore_terminal()

    }
    fn on_tick(&mut self, tree: &mut BonsaiTree, live: bool) {
        let all_changes = tree.animation_step();

        if live {
            self.tree_points.coords = process(self.tree_points.coords, &all_changes, PointType::filter_tree);

            self.leaf_points.coords = process(self.leaf_points.coords, &all_changes, PointType::filter_leaf);
        }

        self.particles.coords = Box::leak(filter(&all_changes, PointType::filter_particles).into_boxed_slice());
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

fn process<'a, 'b, F>(coords: &'a [(f64, f64)], changes: &'b Vec <PointType>, f: F) -> &'a [(f64, f64)] 
    where F: FnMut(&PointType) -> Option<(f64, f64)> {
        let new_coords = filter(changes, f);

        Box::leak([coords, Box::leak(new_coords.into_boxed_slice())].concat().into_boxed_slice())
}

fn filter<F>(changes: &Vec <PointType>, f: F) -> Vec <(f64, f64)> 
    where F: FnMut(&PointType) -> Option<(f64, f64)> {
        changes.iter()
            .filter_map(f)
            .collect()
}
