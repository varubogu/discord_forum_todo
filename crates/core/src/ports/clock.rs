//! `Clock` port plus an always-available `FakeClock` for tests.

use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

mod fake {
    use super::Clock;
    use chrono::{DateTime, Duration, Utc};
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct FakeClock {
        inner: Arc<Mutex<DateTime<Utc>>>,
    }

    impl FakeClock {
        pub fn new(now: DateTime<Utc>) -> Self {
            Self {
                inner: Arc::new(Mutex::new(now)),
            }
        }

        pub fn advance(&self, by: Duration) {
            let mut guard = self.inner.lock().expect("FakeClock mutex poisoned");
            *guard += by;
        }
    }

    impl Clock for FakeClock {
        fn now(&self) -> DateTime<Utc> {
            *self.inner.lock().expect("FakeClock mutex poisoned")
        }
    }
}

pub use fake::FakeClock;
