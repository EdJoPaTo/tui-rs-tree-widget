use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::{Frame, Terminal};
use tui_tree_widget::{Node, Tree, TreeData, TreeState};

struct FileTreeData(PathBuf);

impl TreeData for FileTreeData {
    type Identifier = PathBuf;

    fn get_nodes(
        &self,
        open_identifiers: &HashSet<Self::Identifier>,
    ) -> Vec<Node<Self::Identifier>> {
        let mut result = Vec::new();
        get_nodes_recursive(&mut result, open_identifiers, &self.0, 0);
        result
    }

    fn render(
        &self,
        identifier: &Self::Identifier,
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
    ) {
        let mut spans = Vec::new();
        if let Some(filename) = identifier.file_stem() {
            spans.push(Span::raw(filename.to_string_lossy()));
        }
        if let Some(extension) = identifier.extension() {
            const STYLE: Style = Style::new().fg(Color::DarkGray);
            spans.push(Span::styled(".", STYLE));
            spans.push(Span::styled(extension.to_string_lossy(), STYLE));
        }

        ratatui::widgets::Widget::render(Line::from(spans), area, buffer);
    }
}

/// Collect all the (opened) filesystem entries.
fn get_nodes_recursive(
    result: &mut Vec<Node<PathBuf>>,
    open_identifiers: &HashSet<PathBuf>,
    path: &Path,
    depth: usize,
) {
    let Ok(entries) = path.read_dir() else {
        return;
    };

    // Ignore errors. A real world file viewer should handle them.
    let entries = entries.flatten();

    for entry in entries {
        let is_dir = entry.metadata().is_ok_and(|metadata| metadata.is_dir());
        let path = entry.path();

        result.push(Node {
            depth,
            has_children: is_dir,
            height: 1,
            identifier: path.clone(),
        });

        if open_identifiers.contains(&path) {
            get_nodes_recursive(result, open_identifiers, &path, depth + 1);
        }
    }
}

struct App {
    state: TreeState<PathBuf>,
}

impl App {
    fn new() -> Self {
        Self {
            state: TreeState::default(),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let data = FileTreeData(Path::new(".").to_owned());
        let widget = Tree::new(&data)
            .block(
                Block::bordered()
                    .title("File Tree")
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
