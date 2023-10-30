mod util;

use crate::util::StatefulTree;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Terminal,
};
use std::error::Error;
use std::io;

use tui_tree_widget::{Tree, TreeItem};

struct App<'a> {
    tree: StatefulTree<'a>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            tree: StatefulTree::with_items(vec![
                TreeItem::new_leaf("a", "Alfa"),
                TreeItem::new(
                    "b",
                    "Bravo",
                    vec![
                        TreeItem::new_leaf("c", "Charlie"),
                        TreeItem::new(
                            "d",
                            "Delta",
                            vec![
                                TreeItem::new_leaf("e", "Echo"),
                                TreeItem::new_leaf("f", "Foxtrot"),
                            ],
                        )
                        .expect("all item identifiers are unique"),
                        TreeItem::new_leaf("g", "Golf"),
                    ],
                )
                .expect("all item identifiers are unique"),
                TreeItem::new_leaf("h", "Hotel"),
            ]),
        }
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
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let area = f.size();

            let items = Tree::new(app.tree.items.clone())
                .expect("all item identifiers are unique")
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .title(format!("Tree Widget {:?}", app.tree.state)),
                )
                .highlight_style(
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(items, area, &mut app.tree.state);
        })?;

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
