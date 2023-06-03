use chrono::Utc;
use std::sync::{Arc, Mutex};

pub struct Snowflake {
    pub epoch: i64,
    pub worker_id: i64,
    pub datacenter_id: i64,
    pub sequence: i64,
    pub time: Arc<Mutex<i64>>,
}

impl Snowflake {
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

impl Default for Snowflake {
    fn default() -> Snowflake {
        Snowflake {
            epoch: 1420070400000,
            worker_id: 1,
            datacenter_id: 1,
            sequence: 0,
            time: Arc::new(Mutex::new(0)),
        }
    }
}