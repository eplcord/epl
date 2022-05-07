use chrono::Utc;
use std::sync::{Arc, Mutex};

pub struct Snowflake {
    epoch: i64,
    worker_id: i64,
    datacenter_id: i64,
    sequence: i64,
    time: Arc<Mutex<i64>>,
}

impl Snowflake {
    /// Default configuration for a Snowflake
    pub fn default() -> Snowflake {
        Snowflake {
            epoch: 1420070400000,
            worker_id: 1,
            datacenter_id: 1,
            sequence: 0,
            time: Arc::new(Mutex::new(0)),
        }
    }

    /// Creates a new Snowflake with the provided configuration
    pub fn new(epoch: i64, worker_id: i64, datacenter_id: i64) -> Snowflake {
        Snowflake {
            epoch,
            worker_id,
            datacenter_id,
            sequence: 0,
            time: Arc::new(Mutex::new(0)),
        }
    }

    pub fn epoch(&mut self, epoch: i64) -> &mut Self {
        self.epoch = epoch;
        self
    }

    pub fn worker_id(&mut self, worker_id: i64) -> &mut Self {
        self.worker_id = worker_id;
        self
    }

    pub fn datacenter_id(&mut self, datacenter_id: i64) -> &mut Self {
        self.datacenter_id = datacenter_id;
        self
    }

    /// Generate a new Snowflake
    pub fn generate(&mut self) -> i64 {
        let mut last_timestamp = self.time.lock().expect("Snowflake MutexGuard panic!");
        let mut timestamp = self.get_time();
        if timestamp == *last_timestamp {
            self.sequence = (self.sequence + 1) & (-1 ^ (-1 << 12));
            if self.sequence == 0 && timestamp <= *last_timestamp {
                timestamp = self.get_time();
            }
        } else {
            self.sequence = 0;
        }
        *last_timestamp = timestamp;
        (timestamp << 22) | (self.worker_id << 17) | (self.datacenter_id << 12) | self.sequence
    }

    fn get_time(&self) -> i64 {
        Utc::now().timestamp_millis() - self.epoch
    }
}