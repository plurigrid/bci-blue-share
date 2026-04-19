use color_eyre::eyre::Result;
use std::collections::HashMap;

use crate::api::{Agg, Item};

pub struct App {
    pub endpoint: String,
    pub items: Vec<Item>,
    pub aggs: HashMap<String, Agg>,
    pub selected: usize,
    pub history: Vec<i64>,
    pub last_msg: String,
}

impl App {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            items: Vec::new(),
            aggs: HashMap::new(),
            selected: 0,
            history: vec![0; 60],
            last_msg: String::new(),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        match crate::api::fetch_catalog(&self.endpoint) {
            Ok(items) => self.items = items,
            Err(e) => self.last_msg = format!("catalog err: {}", e),
        }
        match crate::api::fetch_agg(&self.endpoint) {
            Ok(a) => {
                self.aggs = a;
                self.push_history();
            }
            Err(e) => self.last_msg = format!("agg err: {}", e),
        }
        if self.selected >= self.items.len() && !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }
        Ok(())
    }

    fn push_history(&mut self) {
        let net: i64 = self
            .aggs
            .values()
            .map(|x| x.blue as i64 - x.red as i64)
            .sum();
        self.history.push(net);
        while self.history.len() > 120 {
            self.history.remove(0);
        }
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }
    pub fn prev(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn vote(&mut self, choice: i8) -> Result<()> {
        let Some(item) = self.items.get(self.selected).cloned() else {
            return Ok(());
        };
        match crate::api::cast_vote(&self.endpoint, &item.id, choice) {
            Ok(agg) => {
                self.aggs.insert(item.id.clone(), agg);
                self.push_history();
                self.last_msg = format!("voted {} on {}", crate::trit::label(choice), item.id);
            }
            Err(e) => self.last_msg = format!("vote err: {}", e),
        }
        Ok(())
    }

    pub fn totals(&self) -> Agg {
        self.aggs.values().fold(Agg::default(), |mut acc, a| {
            acc.red += a.red;
            acc.blue += a.blue;
            acc.abstain += a.abstain;
            acc.total += a.total;
            acc
        })
    }

    pub fn gf3(&self) -> i64 {
        let t = self.totals();
        let net = t.blue as i64 - t.red as i64;
        ((net % 3) + 3) % 3
    }
}
