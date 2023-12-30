#![allow(non_snake_case)]

mod model;
mod event;
mod update;
mod tui;
mod ui;

use anyhow::Result; 
use sqlx::{ Row, Connection };
use ratatui::{ backend::CrosstermBackend, Terminal };
use model::{App, InputMode};
use update::update;
use event::{ Event, EventHandler };
use tui::Tui;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    let url = "postgres://postgres:postgresPass@localhost:5432/bank";
    let pool = sqlx::postgres::PgPool::connect(url).await?;

    let app = App::new();
    let mut app_arc = Arc::new(Mutex::new(app));

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250, &app_arc);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    /*let res = sqlx::query("SELECT 1 + 1 as sum")
        .fetch_one(&pool)
        .await?;*/

    //let sum: i32 = res.get("sum");

    //println!("{}", sum);
    app_arc.lock().unwrap().change_screen(model::Screen::Login);

    let mut counter = 0;

    while !app_arc.lock().unwrap().should_quit {
        tui.draw(&mut app_arc)?;
        if let Ok(event) = tui.events.next() {
            //println!("{}", counter);
            //counter += 1;
            update(&mut app_arc, &pool, event).await.unwrap_or_else(|error| panic!("{}", error));
        }
    }

    tui.exit()?;

    Ok(())
}