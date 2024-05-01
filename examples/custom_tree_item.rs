use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::{Frame, Terminal};
use tui_tree_widget::{Tree, TreeItem, TreeState};

struct MyTreeItem<'content> {
    identifier: String,
    content: &'content str,
    children: Vec<Self>,
}

impl TreeItem for MyTreeItem<'_> {
    type Identifier = String;

    fn children(&self) -> &[Self] {
        &self.children
    }

    fn height(&self) -> usize {
        // We know that every item will have exactly the height 1
        1
    }

    fn identifier(&self) -> &Self::Identifier {
        &self.identifier
    }

    fn render(&self, area: ratatui::layout::Rect, buffer: &mut ratatui::buffer::Buffer) {
        let line = ratatui::text::Line::raw(self.content);
        ratatui::widgets::Widget::render(line, area, buffer);
    }
}

impl<'content> MyTreeItem<'content> {
    fn new_leaf(identifier: &str, content: &'content str) -> Self {
        Self {
            identifier: identifier.to_owned(),
            content,
            children: Vec::new(),
        }
    }

    fn new(identifier: &str, content: &'content str, children: Vec<Self>) -> Self {
        tui_tree_widget::unique_identifiers::children(&children)
            .expect("all item identifiers are unique");
        Self {
            identifier: identifier.to_owned(),
            content,
            children,
        }
    }

    fn example() -> Vec<Self> {
        vec![
            Self::new_leaf("a", "Alfa"),
            Self::new(
                "b",
                "Bravo",
                vec![
                    Self::new_leaf("c", "Charlie"),
                    Self::new(
                        "d",
                        "Delta",
                        vec![Self::new_leaf("e", "Echo"), Self::new_leaf("f", "Foxtrot")],
                    ),
                    Self::new_leaf("g", "Golf"),
                ],
            ),
            Self::new_leaf("h", "Hotel"),
            Self::new(
                "i",
                "India",
                vec![
                    Self::new_leaf("j", "Juliett"),
                    Self::new_leaf("k", "Kilo"),
                    Self::new_leaf("l", "Lima"),
                    Self::new_leaf("m", "Mike"),
                    Self::new_leaf("n", "November"),
                ],
            ),
            Self::new_leaf("o", "Oscar"),
            Self::new(
                "p",
                "Papa",
                vec![
                    Self::new_leaf("q", "Quebec"),
                    Self::new_leaf("r", "Romeo"),
                    Self::new_leaf("s", "Sierra"),
                    Self::new_leaf("t", "Tango"),
                    Self::new_leaf("u", "Uniform"),
                    Self::new(
                        "v",
                        "Victor",
                        vec![
                            Self::new_leaf("w", "Whiskey"),
                            Self::new_leaf("x", "Xray"),
                            Self::new_leaf("y", "Yankee"),
                        ],
                    ),
                ],
            ),
            Self::new_leaf("z", "Zulu"),
        ]
    }
}

struct App {
    state: TreeState<String>,
}

impl App {
    fn new() -> Self {
        Self {
            state: TreeState::default(),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let items = MyTreeItem::example();
        let widget = Tree::new(items)
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
                        x: frame.size().width.saturating_sub(text.len() as u16),
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
