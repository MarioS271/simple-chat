// SPDX-License-Identifier: GPL-3.0-only
//! Main TUI Logic
//!
//! Authors: MarioS271

use crate::{HEARTBEAT_INTERVAL, MAX_MESSAGE_LEN};
use crate::client::helpers::sender_name_string_to_bytes;
use crate::client::net::receive::receive_thread;
use crate::client::net::send::{send_message, send_packet};
use crate::client::state::ClientState;
use crate::encrypt::EncryptionKey;
use crate::packets::message::formatted::Formatted;
use crate::packets::packet::Packet;
use ratatui::crossterm;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Constraint;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

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

    let state_recv = Arc::clone(&state);
    let read_stream = stream.try_clone()?;
    let write_stream = Arc::new(Mutex::new(stream));
    let heartbeat_stream = Arc::clone(&write_stream);

    std::thread::spawn(move || receive_thread(read_stream, state_recv, encryption_key));
    std::thread::spawn(move || loop {
        std::thread::sleep(HEARTBEAT_INTERVAL);
        if send_packet(&heartbeat_stream.lock().unwrap(), Packet::Heartbeat, encryption_key).is_err() {
            break;
        }
    });

    let sender_name_array = sender_name_string_to_bytes(&state.lock().unwrap().name);

    loop {
        terminal.draw(|frame| {
            render_tui(frame, &state.lock().unwrap());
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            match crossterm::event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
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
                            send_message(
                                &write_stream.lock().unwrap(),
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
            Packet::ChatMessage(inner) => Line::from(
                format!(
                    "[{}] {}: {}",
                    inner.formatted_timestamp(),
                    inner.formatted_sender_name(),
                    inner.formatted_content()
                )
            ),
            Packet::SystemMessage(inner) => Line::from(
                format!(
                    "[{}] {}",
                    inner.formatted_timestamp(),
                    inner.formatted_content()
                )
            ),
            _ => {
                end_raw_mode();
                eprintln!("Attempted to render non-message packet as a message in the UI");
                std::process::exit(1);
            }
        }
    }).collect();

    frame.render_widget(
        Paragraph::new(format!("simple-chat | Connected to {} as {}", state.remote, state.name))
            .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::REVERSED)),
        areas[0]
    );

    let messages_block = ratatui::widgets::Block::bordered().title("Messages");
    let inner_area = messages_block.inner(areas[1]);

    let start = msg_lines.len().saturating_sub(inner_area.height as usize);
    let visible_lines = msg_lines[start..].to_vec();

    let messages = Paragraph::new(visible_lines)
        .wrap(ratatui::widgets::Wrap { trim: false });
    let wrapped_height = messages.line_count(inner_area.width);
    let scroll = wrapped_height.saturating_sub(inner_area.height as usize);

    frame.render_widget(
        messages
            .scroll((scroll.min(u16::MAX as usize) as u16, 0))
            .block(messages_block),
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
