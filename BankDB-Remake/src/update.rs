//use crossterm::event::{Event as CrosstermEvent, KeyEventKind, KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::Event as CrosstermEvent;
use std::sync::{Arc, Mutex};
use tui_input::backend::crossterm::EventHandler;
use sqlx::{Row, Pool, Postgres};
use anyhow::Result;

use crate::model::{App, InputMode, TimeoutType};
use crate::event::Event;

pub async fn update(app: &mut Arc<Mutex<App>>, pool: &Pool<Postgres>, event: Event) -> Result<()> {
    match event {
        Event::Quit => {
            app.lock().unwrap().should_quit = true;
            Ok(())
        },
        Event::TimeoutStep(timeout_type) => {
            app.lock().unwrap().update_timeout_counter(timeout_type);
            Ok(())
        },
        Event::TryLogin => {
            if app.lock().unwrap().failed_logins == 3 {
                return Ok(());
            }

            let name: String = app.lock().unwrap().input.0.value().to_string();
            let password: String = app.lock().unwrap().input.1.value().to_string();

            if let Some(res) = sqlx::query("SELECT name, password FROM clients WHERE LOWER(name) = LOWER($1)")
                .bind(&name)
                .fetch_optional(pool)
                .await? {
                    let res_name: String = res.try_get("name")?;
                    let res_password: String = res.try_get("password")?;
                    if password == res_password {
                        todo!("[ LOGIN SUCCESSFUL ] Name: {res_name}, Password: {res_password}");
                        return Ok(());
                    }
                }
            
            let mut app_lock = app.lock().unwrap();
            app_lock.failed_logins += 1;
            
            if app_lock.failed_logins == 3 {
                app_lock.add_timeout(30, 1000, TimeoutType::Login);
            }
            Ok(())
        },
        Event::SwitchInput => {
            let mut app_lock = app.lock().unwrap();
            if let InputMode::Editing(field) = app_lock.input_mode {
                if field == 0 { app_lock.input_mode = InputMode::Editing(1) }
                else { app_lock.input_mode = InputMode::Editing(0) }
            }
            Ok(())
        }
        Event::Key(key_event) => {
            let mut app_lock = app.lock().unwrap();
            if let InputMode::Editing(field) = app_lock.input_mode {
                if field == 0 { app_lock.input.0.handle_event(&CrosstermEvent::Key(key_event)); }
                else { app_lock.input.1.handle_event(&CrosstermEvent::Key(key_event)); }
            }
            Ok(())
        },
        _ => { Ok(()) }
    }
}