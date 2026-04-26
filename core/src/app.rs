use std::collections::HashMap;
use crate::types::{label, Agg, Item};

pub struct App {
    pub items: Vec<Item>,
    pub aggs: HashMap<String, Agg>,
    pub selected: usize,
    pub history: Vec<i64>,
    pub last_msg: String,
}

impl Default for App { fn default() -> Self { Self::new() } }

impl App {
    pub fn new() -> Self {
        Self { items: Vec::new(), aggs: HashMap::new(), selected: 0, history: vec![0; 60], last_msg: String::new() }
    }
    pub fn set_items(&mut self, items: Vec<Item>) {
        self.items = items;
        if self.selected >= self.items.len() && !self.items.is_empty() { self.selected = self.items.len() - 1; }
    }
    pub fn set_aggs(&mut self, aggs: HashMap<String, Agg>) { self.aggs = aggs; self.push_history(); }
    pub fn apply_vote(&mut self, item: &str, agg: Agg, choice: i8) {
        self.aggs.insert(item.to_string(), agg);
        self.push_history();
        self.last_msg = format!("voted {} on {}", label(choice), item);
    }
    pub fn err(&mut self, msg: String) { self.last_msg = msg; }
    pub fn next(&mut self) { if !self.items.is_empty() { self.selected = (self.selected + 1) % self.items.len(); } }
    pub fn prev(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 { self.items.len() - 1 } else { self.selected - 1 };
        }
    }
    pub fn selected_item(&self) -> Option<&Item> { self.items.get(self.selected) }
    pub fn totals(&self) -> Agg {
        self.aggs.values().fold(Agg::default(), |mut a, x| { a.red+=x.red; a.blue+=x.blue; a.abstain+=x.abstain; a.total+=x.total; a })
    }
    pub fn gf3(&self) -> i64 {
        let t = self.totals();
        let net = t.blue as i64 - t.red as i64;
        ((net % 3) + 3) % 3
    }
    fn push_history(&mut self) {
        let net: i64 = self.aggs.values().map(|x| x.blue as i64 - x.red as i64).sum();
        self.history.push(net);
        while self.history.len() > 120 { self.history.remove(0); }
    }
}
