use ratatui::{
    layout::{Layout, Direction, Rect, Constraint, Margin},
    prelude::{Alignment, Frame},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, List, BorderType, Borders, Paragraph, Clear}
};
use std::sync::{Arc, Mutex};
use crate::{
    model::{
        common::{Popup, InputMode},
        app::App,
    },
    ui::common_fn::{
        centered_rect,
        percent_x,
        percent_y,
        clear_chunks
    }
};

pub fn render(app: &mut Arc<Mutex<App>>, f: &mut Frame) {
    let mut app_lock = app.lock().unwrap();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Percentage(100),
            Constraint::Min(2)
        ])
        .split(centered_rect(
            percent_x(f, 2.0),
            percent_y(f, 1.5),
            f.size()));
    
    if app_lock.should_clear_screen {
        clear_chunks(f, &chunks);
        app_lock.should_clear_screen = false;
    }

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])       
        .split(chunks[0]);
    
    let header_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded); 

    let header_login = Paragraph::new(Text::from(
        format!("\n  Login: {}", app_lock.client.active.as_ref().unwrap().name
    )));
    
    let header_balance = Paragraph::new(Text::from(
        format!("\nBalance: {}$  ", app_lock.client.active.as_ref().unwrap().balance
    ))).alignment(Alignment::Right);

    f.render_widget(header_login, header_chunks[0]);
    f.render_widget(header_balance, header_chunks[1]);
    f.render_widget(header_block, chunks[0]); 

    let list_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(5),
            Constraint::Percentage(90),
            Constraint::Percentage(5)
        ])
        .split(chunks[1]);
    
    let actions_text: Vec<String> = app_lock.client.actions.iter().map(|action| action.to_list_string().to_string()).collect();

    let actions = List::new(actions_text).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    f.render_stateful_widget(actions, list_chunks[1], &mut app_lock.client.actions_list_state);

    let help_text = Paragraph::new(Text::from(
        format!("{}", app_lock.help_text)
    ))
    .block(Block::default().borders(Borders::TOP));

    f.render_widget(help_text, chunks[2]);

    match app_lock.active_popup {
        Some(Popup::ViewInfo) => {
            let popup_rect = centered_rect(
                percent_x(f, 1.0),
                percent_y(f, 1.0),
                f.size()
            );

            f.render_widget(Clear, popup_rect);
            
            let client_info_block = Block::default().borders(Borders::ALL).border_type(BorderType::QuadrantOutside);

            let active_user = app_lock.client.active.as_ref().unwrap();
            let client_info = Paragraph::new(vec![
                Line::from(Span::raw("Client Information")),
                Line::default(),
                Line::from(Span::raw(format!("Full name: {}", active_user.name))),
                Line::from(Span::raw(format!("C.I.: {}", active_user.ci))),
                Line::from(Span::raw(format!("Account Num.: {}", active_user.account_number))),
                Line::from(Span::raw(format!("Account Type: {:?}", active_user.account_type))),
                Line::from(Span::raw(format!("Balance: {}$", active_user.balance)))
            ])
            .alignment(Alignment::Center)
            .block(client_info_block);

            f.render_widget(client_info, popup_rect);
        },
        Some(Popup::Deposit) | Some(Popup::Withdraw) => {
            let popup_rect = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Percentage(100)
                ]).split(centered_rect(
                percent_x(f, 1.0),
                percent_y(f, 0.4),
                f.size())
            );

            f.render_widget(Clear, popup_rect[0]);

            let title = {
                if let Some(Popup::Deposit) = app_lock.active_popup { "Deposit amount" }
                else { "Withdraw amount" }
            };

            let deposit_block = Block::default().borders(Borders::ALL).border_type(BorderType::Thick).title(title);

            let deposit = Paragraph::new(Line::from(vec![
                Span::raw(" "),
                Span::raw(app_lock.input.0.value()),
            ]))
            .block(deposit_block)
            .alignment(Alignment::Left);

            f.render_widget(deposit, popup_rect[0]);

            f.set_cursor(
                popup_rect[0].x
                + app_lock.input.0.visual_cursor() as u16
                + 2,
                popup_rect[0].y + 1);
        }
        Some(Popup::Transfer) | Some(Popup::ChangePsswd) => {
            let popup_rect = centered_rect(
                percent_x(f, 1.0),
                percent_y(f, 0.9),
                f.size()
            );

            let popup_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(1),
                    Constraint::Min(3),
                ])
                .split(popup_rect.inner(&Margin::new(1, 1)));
            
            let popup_block_title: &str;
            let upper_block_title: &str;
            let lower_block_title: &str;
            
            if let Some(Popup::Transfer) = app_lock.active_popup {
                popup_block_title = "Transfer";
                upper_block_title = "Amount";
                lower_block_title = "Beneficiary";
            } else {
                popup_block_title = "Change Password";
                upper_block_title = "Current Password";
                lower_block_title = "New Password";
            }

            let popup_block = Block::default().borders(Borders::ALL).border_type(BorderType::Thick).title(popup_block_title);
            let amount_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(upper_block_title);
            let beneficiary_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(lower_block_title);

            let amount = Paragraph::new(Line::from(vec![
                Span::raw(" "),
                Span::raw(app_lock.input.0.value())
            ]))
            .block(amount_block);
            
            let beneficiary = Paragraph::new(Line::from(vec![
                Span::raw(" "),
                Span::raw(app_lock.input.1.value())
            ]))
            .block(beneficiary_block);

            if let InputMode::Editing(field) = app_lock.input_mode {
                if field == 0 {
                    f.set_cursor(popup_chunks[0].x
                                    + app_lock.input.0.visual_cursor() as u16
                                    + 2,
                                popup_chunks[0].y + 1,
                                );
                } else {
                    f.set_cursor(popup_chunks[2].x
                                    + app_lock.input.1.visual_cursor() as u16
                                    + 2,
                                popup_chunks[2].y + 1,
                                );
                }
            }
            
            f.render_widget(Clear, popup_rect);
            f.render_widget(popup_block, popup_rect);
            f.render_widget(amount, popup_chunks[0]);
            f.render_widget(beneficiary, popup_chunks[2]);
        }
        _ => {}
    }
}