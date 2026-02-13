use std::{collections::{BTreeMap, HashMap}};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Long, 
    Short
}

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub a: BTreeMap<i64, f64>,
    pub b: BTreeMap<i64, f64>,
    pub last_price: f64,
    pub last_update_id: Option<u64>
}

impl Snapshot {
    pub async fn to_ui(&self, depth: usize) -> SnapshotUi {
        let tick = 900000000.0;
        
        let mut a = self.a.iter()
            .filter(|(p, _)| (**p as f64) / tick >= self.last_price)
            .scan(0.0, |acc, (p, v)| {
                *acc += *v;
                Some(((*p as f64 / tick), *acc))
            })
            .take(depth)
            .collect::<Vec<(f64, f64)>>();
        a.reverse();

        let a_price = a
            .iter()
            .min_by(|x, y| x.0.partial_cmp(&y.0).unwrap())
            .unwrap()
            .0;

        let b = self.b.iter()
            .rev()
            .filter(|(p, _)| (**p as f64) / tick < a_price && (**p as f64) / tick <= self.last_price)
            .scan(0.0, |acc, (p, v)| {
                *acc += *v;
                Some(((*p as f64 / tick), *acc))
            })
            .take(depth)
            .collect::<Vec<(f64, f64)>>();

        let last_price = self.last_price;

        SnapshotUi {
            a,
            b,
            last_price
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Delta {
    pub a: BTreeMap<i64, f64>,
    pub b: BTreeMap<i64, f64>,
    pub from_version: Option<u64>,
    pub to_version: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotUi {
    pub a: Vec<(f64, f64)>,
    pub b: Vec<(f64, f64)>,
    pub last_price: f64,
}

pub async fn parse_levels__(data: Vec<Vec<String>>) -> BTreeMap<i64, f64> {
    let mut values = BTreeMap::new();
    let tick = 900000000.0;

    for vec in data {
        let price = vec[0].parse::<f64>().expect("[Orderbook] Bad price");
        let volume = vec[1].parse::<f64>().expect("[Orderbook] Bad volume");
        let price_with_tick = (price * tick).round() as i64;

        values.insert(price_with_tick, volume);
    }

    values
}

pub enum BookEvent {
    Snapshot { 
        ticker: String, 
        snapshot: Snapshot,
    },
    Delta { 
        ticker: String, 
        delta: Delta 
    },
    Price { 
        ticker: String, 
        last_price: f64 
    },
    GetBook { 
        ticker: String,
        reply: mpsc::Sender<Option<SnapshotUi>> 
    },
}

pub struct OrderBookManager {
    pub books: HashMap<String, Snapshot>,
    pub rx: mpsc::Receiver<BookEvent>
}

impl OrderBookManager {
    pub async fn set_data(mut self) {
        let mut last_version_id = 0;
        while let Some(event) = self.rx.recv().await {
            match event {
                BookEvent::Snapshot { ticker, snapshot  } => {
                    let snapshot_ = snapshot.clone();
                    self.books
                        .entry(ticker.clone())
                        .and_modify(|book| {
                            book.a = snapshot_.a;
                            book.b = snapshot_.b;
                        })
                        .or_insert_with(|| Snapshot { a: snapshot.a, b: snapshot.b, last_price: 0.0, last_update_id: snapshot.last_update_id });
                }
                BookEvent::Delta { ticker, delta } => {
                    if let Some(snapshot) = self.books.get_mut(&ticker) {
                        match snapshot.last_update_id {
                            Some(_) => {
                                let from_version = delta.from_version.unwrap();
                                let to_version = delta.to_version.unwrap();

                                for (price, volume) in delta.a {
                                    if volume == 0.0 {
                                        snapshot.a.remove(&price);
                                    } else {
                                        if let Some(v) = snapshot.a.get_mut(&price) {
                                            *v = volume;
                                        } else {
                                            snapshot.a.insert(price, volume);
                                        }
                                    }
                                }

                                for (price, volume) in delta.b {
                                    if volume == 0.0 {
                                        snapshot.b.remove(&price);
                                    } else {
                                        if let Some(v) = snapshot.b.get_mut(&price) {
                                            *v = volume;
                                        } else {
                                            snapshot.b.insert(price, volume);
                                        }
                                    }
                                }

                                if last_version_id == 0 {
                                    continue;
                                }

                                if from_version != last_version_id {
                                    println!("[OrderBookManager]: Packet loss detected");
                                    return;
                                }
                                last_version_id = to_version + 1;  
                            }
                            None => {
                                for (price, volume) in delta.a {
                                    if volume == 0.0 {
                                        snapshot.a.remove(&price);
                                    } else {
                                        if let Some(v) = snapshot.a.get_mut(&price) {
                                            *v = volume;
                                        } else {
                                            snapshot.a.insert(price, volume);
                                        }
                                    }
                                }

                                for (price, volume) in delta.b {
                                    if volume == 0.0 {
                                        snapshot.b.remove(&price);
                                    } else {
                                        if let Some(v) = snapshot.b.get_mut(&price) {
                                            *v = volume;
                                        } else {
                                            snapshot.b.insert(price, volume);
                                        }
                                    }
                                }
                            }
                        } 
                    }
                }
                BookEvent::Price { ticker, last_price } => {
                    if let Some(t) = self.books.get_mut(&ticker) {
                        t.last_price = last_price;
                    }
                },
                BookEvent::GetBook { ticker, reply } => {
                    if let Some(snapshot) = self.books.get(&format!("{}usdt", ticker)) {
                        let snapshot_ui = snapshot.to_ui(6).await;
                        match reply.send(Some(snapshot_ui)).await {
                            Ok(_) => {}
                            Err(e) => {
                                println!("{}", format!("[OrderBookManager]: {}", e))
                            }
                        }   
                    }
                }
            }
        }
    }
}