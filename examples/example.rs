use std::time::{Duration, Instant};

use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::{crossterm, Frame, Terminal};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[must_use]
struct App {
    state: TreeState<&'static str>,
    items: Vec<TreeItem<'static, &'static str>>,
}

impl App {
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

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let widget = Tree::new(&self.items)
            .expect("all item identifiers are unique")
            .block(
                Block::bordered()
                    .title("Tree Widget")
                    .title_bottom(format!("{:?}", self.state)),
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
        frame.render_stateful_widget(widget, area, &mut self.state);
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
    const DEBOUNCE: Duration = Duration::from_millis(20); // 50 FPS

    let before = Instant::now();
    terminal.draw(|frame| app.draw(frame))?;
    let mut last_render_took = before.elapsed();

    let mut debounce: Option<Instant> = None;

    loop {
        let timeout = debounce.map_or(DEBOUNCE, |start| DEBOUNCE.saturating_sub(start.elapsed()));
        if crossterm::event::poll(timeout)? {
            let update = match crossterm::event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('\n' | ' ') => app.state.toggle_selected(),
                    KeyCode::Left => app.state.key_left(),
                    KeyCode::Right => app.state.key_right(),
                    KeyCode::Down => app.state.key_down(),
                    KeyCode::Up => app.state.key_up(),
                    KeyCode::Esc => app.state.select(Vec::new()),
                    KeyCode::Home => app.state.select_first(),
                    KeyCode::End => app.state.select_last(),
                    KeyCode::PageDown => app.state.scroll_down(3),
                    KeyCode::PageUp => app.state.scroll_up(3),
                    _ => false,
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => app.state.scroll_down(1),
                    MouseEventKind::ScrollUp => app.state.scroll_up(1),
                    MouseEventKind::Down(_button) => {
                        app.state.click_at(Position::new(mouse.column, mouse.row))
                    }
                    _ => false,
                },
                Event::Resize(_, _) => true,
                _ => false,
            };
            if update {
                debounce.get_or_insert_with(Instant::now);
            }
        }
        if debounce.is_some_and(|debounce| debounce.elapsed() > DEBOUNCE) {
            let before = Instant::now();
            terminal.draw(|frame| {
                app.draw(frame);

                // Performance info in top right corner
                {
                    let text = format!(
                        " {} {last_render_took:?} {:.1} FPS",
                        frame.count(),
                        1.0 / last_render_took.as_secs_f64()
                    );
                    #[allow(clippy::cast_possible_truncation)]
                    let area = Rect {
                        y: 0,
                        height: 1,
                        x: frame.area().width.saturating_sub(text.len() as u16),
                        width: text.len() as u16,
                    };
                    frame.render_widget(
                        Span::styled(text, Style::new().fg(Color::Black).bg(Color::Gray)),
                        area,
                    );
                }
            })?;
            last_render_took = before.elapsed();

            debounce = None;
        }
    }
}
