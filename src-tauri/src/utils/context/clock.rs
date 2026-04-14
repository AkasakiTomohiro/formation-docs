use chrono::{DateTime, Utc};

#[mockall::automock]
pub trait Clock: Send + Sync {
    fn utc_now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;
#[coverage(off)]
impl Clock for SystemClock {
    fn utc_now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
