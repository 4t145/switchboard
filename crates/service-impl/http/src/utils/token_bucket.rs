use std::sync::Arc;

use tokio::{
    sync::Mutex,
    time::{Duration, Instant},
};

pub struct TokenBucket {
    pub capacity: usize,
    pub rate: Duration,
    state: Arc<Mutex<TokenBucketState>>,
}

impl TokenBucket {
    pub fn new(capacity: usize, rate: Duration) -> Self {
        let now = Instant::now();
        Self {
            capacity,
            rate,
            state: Arc::new(Mutex::new(TokenBucketState {
                prev_tick: now,
                token_count: capacity,
                last_seen: now,
            })),
        }
    }

    pub async fn require(&self, time: Instant) -> Option<Token> {
        self.state
            .lock()
            .await
            .require(time, self.capacity, self.rate)
    }

    pub async fn idle_for(&self, time: Instant) -> Duration {
        self.state.lock().await.idle_for(time)
    }
}

struct TokenBucketState {
    prev_tick: Instant,
    token_count: usize,
    last_seen: Instant,
}

pub struct Token {
    _priv: (),
}

impl TokenBucketState {
    fn require(&mut self, time: Instant, capacity: usize, rate: Duration) -> Option<Token> {
        let elapsed = time.saturating_duration_since(self.prev_tick);
        let refill_count = elapsed.as_nanos() / rate.as_nanos();
        if refill_count > 0 {
            let refill_count = refill_count.min(usize::MAX as u128) as usize;
            self.token_count = self.token_count.saturating_add(refill_count).min(capacity);
            self.prev_tick += rate.saturating_mul(refill_count as u32);
        }
        self.last_seen = time;
        if self.token_count > 0 {
            self.token_count -= 1;
            Some(Token { _priv: () })
        } else {
            None
        }
    }

    fn idle_for(&self, time: Instant) -> Duration {
        time.saturating_duration_since(self.last_seen)
    }
}
