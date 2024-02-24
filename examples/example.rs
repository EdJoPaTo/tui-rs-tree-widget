use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::Terminal;

use tui_tree_widget::{Tree, TreeItem, TreeState};

struct App<'a> {
    state: TreeState<&'static str>,
    items: Vec<TreeItem<'a, &'static str>>,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            state: TreeState::default(),
            items: vec![
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
                TreeItem::new(
                    "i",
                    "India",
                    vec![
                        TreeItem::new_leaf("j", "Juliett"),
                        TreeItem::new_leaf("k", "Kilo"),
                        TreeItem::new_leaf("l", "Lima"),
                        TreeItem::new_leaf("m", "Mike"),
                        TreeItem::new_leaf("n", "November"),
                    ],
                )
                .expect("all item identifiers are unique"),
                TreeItem::new_leaf("o", "Oscar"),
                TreeItem::new(
                    "p",
                    "Papa",
                    vec![
                        TreeItem::new_leaf("q", "Quebec"),
                        TreeItem::new_leaf("r", "Romeo"),
                        TreeItem::new_leaf("s", "Sierra"),
                        TreeItem::new_leaf("t", "Tango"),
                        TreeItem::new_leaf("u", "Uniform"),
                        TreeItem::new(
                            "v",
                            "Victor",
                            vec![
                                TreeItem::new_leaf("w", "Whiskey"),
                                TreeItem::new_leaf("x", "Xray"),
                                TreeItem::new_leaf("y", "Yankee"),
                            ],
                        )
                        .expect("all item identifiers are unique"),
                    ],
                )
                .expect("all item identifiers are unique"),
                TreeItem::new_leaf("z", "Zulu"),
            ],
        }
    }
}

fn main() -> std::io::Result<()> {
    // Terminal initialization
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // App
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            let widget = Tree::new(app.items.clone())
                .expect("all item identifiers are unique")
                .block(
                    Block::bordered()
                        .title("Tree Widget")
                        .title_bottom(format!("{:?}", app.state)),
                )
                .experimental_scrollbar(Some(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(None)
                        .track_symbol(None)
                        .end_symbol(None),
                ))
                .highlight_style(
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            frame.render_stateful_widget(widget, area, &mut app.state);
        })?;

        match crossterm::event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('\n' | ' ') => app.state.toggle_selected(),
                KeyCode::Left => app.state.key_left(),
                KeyCode::Right => app.state.key_right(),
                KeyCode::Down => app.state.key_down(&app.items),
                KeyCode::Up => app.state.key_up(&app.items),
                KeyCode::Home => {
                    app.state.select_first(&app.items);
                }
                KeyCode::End => {
                    app.state.select_last(&app.items);
                }
                KeyCode::PageDown => app.state.scroll_down(3),
                KeyCode::PageUp => app.state.scroll_up(3),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => app.state.scroll_down(1),
                MouseEventKind::ScrollUp => app.state.scroll_up(1),
                _ => {}
            },
            _ => {}
        }
    }
}
