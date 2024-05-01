use std::process::Command;

use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Scrollbar, ScrollbarOrientation};
use ratatui::{Frame, Terminal};
use serde_json::Value;
use tui_tree_widget::{Selector, Tree, TreeState};

struct App {
    metadata: Value,
    state: TreeState<Selector>,
}

impl App {
    fn new() -> Self {
        let output = Command::new("cargo")
            .arg("metadata")
            .arg("--format-version=1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "cargo metadata should be executed successfully"
        );
        let stdout = String::from_utf8(output.stdout).expect("Should be able to parse metadata");
        let metadata: Value = serde_json::from_str(&stdout).unwrap();
        Self {
            metadata,
            state: TreeState::default(),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let widget = Tree::new(tui_tree_widget::json::tree_items(&self.metadata))
            .expect("JSON Should always have unique identifiers")
            .block(
                Block::bordered()
                    .title("cargo metadata (run with --release for best results)")
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
    terminal.draw(|frame| app.draw(frame))?;
    loop {
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
            _ => false,
        };
        if update {
            terminal.draw(|frame| app.draw(frame))?;
        }
    }
}
