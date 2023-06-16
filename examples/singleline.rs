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
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Terminal,
};

use tui_tree_widget::{Tree, TreeItem};

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
        let mut tree = StatefulTree::with_items(vec![
            TreeItem::new_leaf("a"),
            TreeItem::new(
                "b",
                vec![
                    TreeItem::new_leaf("c"),
                    TreeItem::new("d", vec![TreeItem::new_leaf("e"), TreeItem::new_leaf("f")]),
                    TreeItem::new_leaf("g"),
                ],
            ),
            TreeItem::new_leaf("h"),
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

            let items = Tree::new(app.tree.items.clone())
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "Tree Widget {:?} {:?}",
                    app.tree.state, app.performance
                )))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
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
