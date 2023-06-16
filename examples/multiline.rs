mod util;

use crate::util::StatefulTree;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, time::SystemTime};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Terminal,
};

use tui_tree_widget::{MultilineTree, TreeItem};

const P_G: &str = "Peer selection when creating outbound connections should be based on a *similarity index* based on how many tracked projects are shared/in common.

Once #25 is done, the next iteration is to be smart about who we connect to. To compute the similarity index between our node and a peer, we can compare our bloom filter with a peer's bloom filter, using the built-in similarity metrics of the bloom filter implementation.

We then rank all peers by similarity, and connect to the closest peers.";
const P_H: &str = "We've implemented a temporary fix for the missing `WindowResize` in `tuirealms
backend adapter for `termion`. We should add this to upstream.

│We could create a thread that listens on signals and then send over a channel to
the UI thread:
```
pub fn signals(channel: chan::Sender<Signal>) -> Result<(), Error> {
    use signal_hook::consts::signal::*;
    let mut signals = signal_hook::iterator::Signals::new([SIGWINCH, SIGINT])?;
    for signal in signals.forever() {
        match signal {
            SIGWINCH => channel.send(Signal::WindowResized)?,
            SIGINT => channel.send(Signal::Interrupted)?,
            _ => {}
        }
    }
    Ok(())
}
```";

#[derive(Debug)]
struct Performance {
    pub render_time: u128,
}

struct App<'a> {
    tree: StatefulTree<'a>,
    performance: Performance,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let constraints = vec![Constraint::Length(1), Constraint::Max(8)];

        let mut tree = StatefulTree::with_items(vec![
            TreeItem::new_leaf("# A"),
            TreeItem::new(
                "# B",
                vec![
                    TreeItem::new_leaf("# C"),
                    TreeItem::new(
                        "# D",
                        vec![TreeItem::new_leaf("# E"), TreeItem::new_leaf("# F")],
                    ),
                    TreeItem::new_leaf("# G")
                        .paragraph(P_G)
                        .heights(&constraints),
                ],
            ),
            TreeItem::new_leaf("# H")
                .paragraph(P_H)
                .heights(&constraints),
        ]);
        tree.first();

        let performance = Performance { render_time: 0 };
        Self { tree, performance }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        let now = SystemTime::now();
        terminal.draw(|f| {
            let area = f.size();

            let items = MultilineTree::new(app.tree.items.clone())
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "Tree Widget {:?} {:?}",
                    app.tree.state, app.performance
                )))
                .item_block(Block::default().borders(Borders::ALL))
                .item_block_highlight(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::LightGreen)),
                )
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            f.render_stateful_widget(items, area, &mut app.tree.state);
        })?;

        let elapsed = now.elapsed().unwrap_or_default();
        app.performance.render_time = elapsed.as_millis();

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('\n' | ' ') => app.tree.toggle(),
                KeyCode::Left => app.tree.left(),
                KeyCode::Right => app.tree.right(),
                KeyCode::Down => app.tree.down(),
                KeyCode::Up => app.tree.up(),
                KeyCode::Home => app.tree.first(),
                KeyCode::End => app.tree.last(),
                _ => {}
            }
        }
    }
}
