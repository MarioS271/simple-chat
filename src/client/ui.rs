// SPDX-License-Identifier: GPL-3.0-only
//! Main TUI Logic
//!
//! Authors: MarioS271

use crate::client::helpers::sender_name_string_to_bytes;
use crate::client::net::receive::receive_thread;
use crate::client::state::ClientState;
use crate::encrypt::EncryptionKey;
use crate::message::{Formatted, Message};
use ratatui::crossterm;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Constraint;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

const MAX_MESSAGE_LEN: usize = 512;

pub fn start_tui(state: Arc<Mutex<ClientState>>, stream: TcpStream, encryption_key: EncryptionKey) -> std::io::Result<()> {
    let _guard = TerminalGuard;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen
    )?;

    let mut terminal = ratatui::Terminal::new(
        ratatui::backend::CrosstermBackend::new(std::io::stdout())
    )?;

    let read_stream = stream.try_clone()?;
    let state_recv = Arc::clone(&state);

    std::thread::spawn(move || receive_thread(read_stream, state_recv, encryption_key));

    let sender_name_array = sender_name_string_to_bytes(&state.lock().unwrap().name);

    loop {
        terminal.draw(|frame| {
            render_tui(frame, &state.lock().unwrap());
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            match crossterm::event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char(c) => {
                            let mut state = state.lock().unwrap();
                            if state.input.len() < MAX_MESSAGE_LEN {
                                state.input.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            state.lock().unwrap().input.pop();
                        }
                        KeyCode::Enter => {
                            let input = {
                                let mut state = state.lock().unwrap();
                                let msg = state.input.trim().to_string();
                                state.input.clear();
                                msg
                            };
                            crate::client::net::send::send(
                                &stream,
                                sender_name_array,
                                input,
                                encryption_key
                            )?;
                        }
                        KeyCode::Esc => {
                            end_raw_mode();
                            std::process::exit(0);
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn render_tui(frame: &mut ratatui::Frame, state: &ClientState) {
    let areas = ratatui::layout::Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(3)
    ]).split(frame.area());

    let msg_lines: Vec<Line> = state.messages.iter().map(|msg| {
        match msg {
            Message::Chat(inner) => Line::from(
                format!(
                    "[{}] {}: {}",
                    inner.formatted_timestamp(),
                    inner.formatted_sender_name(),
                    inner.formatted_content()
                )
            ),
            Message::System(inner) => Line::from(
                format!(
                    "[{}] {}",
                    inner.formatted_timestamp(),
                    inner.formatted_content()
                )
            ),
        }
    }).collect();

    frame.render_widget(
        Paragraph::new(format!("lan-chat | Connected to {} as {}", state.remote, state.name))
            .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::REVERSED)),
        areas[0]
    );

    let visible_height = areas[1].height.saturating_sub(2) as usize;
    let start = msg_lines.len().saturating_sub(visible_height);
    let visible_lines = msg_lines[start..].to_vec();

    frame.render_widget(
        Paragraph::new(visible_lines)
            .block(ratatui::widgets::Block::bordered().title("Messages")),
        areas[1]
    );

    let available_width = areas[2].width.saturating_sub(2) as usize;
    let char_count = state.input.chars().count();
    let display_input = if char_count > available_width {
        let start = state.input
            .char_indices()
            .nth(char_count - available_width)
            .map(|(byte_index, _)| byte_index)
            .unwrap_or(0);
        &state.input[start..]
    } else {
        &state.input
    };

    frame.render_widget(
        Paragraph::new(format!(" {}", display_input))
            .block(ratatui::widgets::Block::bordered().title("Input")),
        areas[2]
    );

    let cursor_x = areas[2].x + 2 + display_input.chars().count() as u16;
    let cursor_y = areas[2].y + 1;
    frame.set_cursor_position(ratatui::layout::Position { x: cursor_x, y: cursor_y });
}

struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        end_raw_mode()
    }
}
pub fn end_raw_mode() {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen
    );
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::Show
    );
}
