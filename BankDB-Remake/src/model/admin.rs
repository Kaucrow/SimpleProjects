use std::{fs, collections::HashMap};
use anyhow::Result;
use sqlx::{FromRow, PgPool};
use ratatui::widgets::{ListState, TableState};
use crate::model::{
    common::{Popup, SideScreen, CltData, Button, ScreenSection},
    client::Client,
};
use crate::DATA_PATH;

pub struct AdminData {
    pub actions: Vec<&'static str>,
    pub action_list_state: ListState,
    pub client_table_state: TableState,
    pub stored_clients: Vec<Client>,
    pub viewing_clients: i32,
    pub query_clients: String,
    pub popups: HashMap<usize, Popup>,
    pub cltdata: Vec<&'static str>,
    pub cltdata_sidescreens: HashMap<usize, CltData>,
    pub cltdata_list_state: ListState,
    pub popup_screen_section: ScreenSection,
    pub button_selection: Option<Button>,
    pub active_cltdata: Option<CltData>,
    pub applied_filters: HashMap<CltData, Option<String>>,
    pub registered_cltdata: HashMap<CltData, Option<String>>,
    pub active_sidescreen: SideScreen,
    pub user_logo: String,
    pub client_edit_fields: Vec<&'static str>
}

impl std::default::Default for AdminData {
    fn default() -> Self {
        AdminData {
            actions: vec![
                "Filter clients",
                "Add a client"
            ],
            action_list_state: ListState::default(),
            client_table_state: TableState::default(),
            stored_clients: Vec::new(),
            viewing_clients: 0,
            query_clients: String::from("SELECT * FROM clients"),
            popups: HashMap::from([
                (0, Popup::FilterClients),
                (1, Popup::AddClient)
            ]),
            cltdata: vec![
                "Username",
                "Name",
                "C.I.",
                "Account number",
                "Balance",
                "Account type",
                "Account status",
            ],
            cltdata_sidescreens: HashMap::from([
                (0, CltData::Username),
                (1, CltData::Name),
                (2, CltData::Ci),
                (3, CltData::AccNum),
                (4, CltData::Balance),
                (5, CltData::AccType),
                (6, CltData::AccStatus),
            ]),
            cltdata_list_state: ListState::default(),
            popup_screen_section: ScreenSection::Left,
            button_selection: None,
            active_cltdata: None,
            applied_filters: HashMap::from([
                (CltData::Username, None),
                (CltData::Name, None),
                (CltData::Ci, None),
                (CltData::AccNum, None),
                (CltData::Balance, None),
                (CltData::AccType, None),
                (CltData::AccStatus, None),
            ]),
            registered_cltdata: HashMap::from([
                (CltData::Username, None),
                (CltData::Name, None),
                (CltData::Ci, None),
                (CltData::AccNum, None),
                (CltData::Balance, None),
                (CltData::AccType, None),
                (CltData::AccStatus, None),
                (CltData::PsswdHash, None),
            ]),
            active_sidescreen: SideScreen::AdminClientTable,
            user_logo: fs::read_to_string(format!("{}user_logo.txt", DATA_PATH.lock().unwrap())).unwrap(),
            client_edit_fields: vec![
                "Username: ",
                "C.I.: ",
                "Account num.: ",
                "Balance: ",
                "Account type: ",
                "Last transaction: ",
                "Account status: ",
            ]
        }
    }
}

pub enum ModifiedTable {
    Yes,
    No
}

pub enum GetClientsType {
    Next,
    Previous
}

#[derive(Debug, PartialEq)]
pub enum CltDataType {
    CltData,
    Filter
}

impl AdminData {
    pub async fn get_clients(&mut self, pool: &PgPool, get_type: GetClientsType) -> Result<ModifiedTable> {
        match get_type {
            GetClientsType::Next => self.viewing_clients += 10,
            GetClientsType::Previous => {
                if self.viewing_clients == 0 { return Ok(ModifiedTable::No); }
                self.viewing_clients -= 10;
            }
        }

        self.query_clients.push_str(format!(" LIMIT 10 OFFSET {}", self.viewing_clients).as_str());
        let result = self.get_clients_raw(pool, false).await?;
        self.query_clients.truncate(self.query_clients.find(" LIMIT").unwrap());

        if let (GetClientsType::Next, ModifiedTable::No) = (get_type, &result) {
            self.viewing_clients += 10
        }

        Ok(result)
    }

    pub async fn get_clients_raw(&mut self, pool: &PgPool, store_if_res_empty: bool) -> Result<ModifiedTable> {
        let res: Vec<Client> = {
            sqlx::query(self.query_clients.as_str())
            .fetch_all(pool)
            .await?
            .iter()
            .map(|row| { Client::from_row(row) } )
            .collect::<Result<_, sqlx::Error>>()?
        };

        if !res.is_empty() || store_if_res_empty {
            self.stored_clients.clear();
            self.stored_clients = res;
            return Ok(ModifiedTable::Yes);
        }

        Ok(ModifiedTable::No)
    }
}