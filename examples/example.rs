use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::{Frame, Terminal};
use tui_tree_widget::{GenericTreeItem, Tree,  TreeState};

const fn nato_phonetic(letter: char) -> Option<&'static str> {
    let word = match letter {
        'a' => "Alfa",
        'b' => "Bravo",
        'c' => "Charlie",
        'd' => "Delta",
        'e' => "Echo",
        'f' => "Foxtrot",
        'g' => "Golf",
        'h' => "Hotel",
        'i' => "India",
        'j' => "Juliett",
        'k' => "Kilo",
        'l' => "Lima",
        'm' => "Mike",
        'n' => "November",
        'o' => "Oscar",
        'p' => "Papa",
        'q' => "Quebec",
        'r' => "Romeo",
        's' => "Sierra",
        't' => "Tango",
        'u' => "Uniform",
        'v' => "Victor",
        'w' => "Whiskey",
        'x' => "Xray",
        'y' => "Yankee",
        'z' => "Zulu",
        _ => return None,
    };
    Some(word)
}

struct Item {
    letter: char,
    children: Vec<Self>,
}

impl Item {
    const fn new_leaf(letter: char) -> Self {
        Self::new(letter, Vec::new())
    }

    const fn new(letter: char, children: Vec<Self>) -> Self {
        Self { letter, children }
    }
}

impl GenericTreeItem for Item {
    type Identifier = char;

    fn identifier(&self) -> &Self::Identifier {
        &self.letter
    }

    fn children(&self) -> &[Self] {
        &self.children
    }

    fn height(&self) -> usize {
        1
    }

    fn render(&self, area: ratatui::layout::Rect, buffer: &mut ratatui::buffer::Buffer) {
        if let Some(word) = nato_phonetic(self.letter) {
            ratatui::widgets::Widget::render(word, area, buffer);
        } else {
            ratatui::widgets::Widget::render(self.letter.to_string(), area, buffer);
        }
    }
}

#[must_use]
struct App {
    state: TreeState<Vec<char>>,
    items: Vec<Item>,
}

impl App {
    fn new() -> Self {
        Self {
            state: TreeState::default(),
            items: vec![
                Item::new_leaf('a'),
                Item::new(
                    'b',
                    vec![
                        Item::new_leaf('c'),
                        Item::new('d', vec![Item::new_leaf('e'), Item::new_leaf('f')]),
                        Item::new_leaf('g'),
                    ],
                ),
                Item::new_leaf('h'),
                Item::new(
                    'i',
                    vec![
                        Item::new_leaf('j'),
                        Item::new_leaf('k'),
                        Item::new_leaf('l'),
                        Item::new_leaf('m'),
                        Item::new_leaf('n'),
                    ],
                ),
                Item::new_leaf('o'),
                Item::new(
                    'p',
                    vec![
                        Item::new_leaf('q'),
                        Item::new_leaf('r'),
                        Item::new_leaf('s'),
                        Item::new_leaf('t'),
                        Item::new_leaf('u'),
                        Item::new(
                            'v',
                            vec![
                                Item::new_leaf('w'),
                                Item::new_leaf('x'),
                                Item::new_leaf('y'),
                            ],
                        ),
                    ],
                ),
                Item::new_leaf('z'),
            ],
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let widget = Tree::new(&self.items)
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
                    KeyCode::Esc => app.state.select(None),
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
