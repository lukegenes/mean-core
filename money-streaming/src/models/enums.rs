#[derive(PartialEq)]
pub enum StreamStatus {
    Scheduled = 0,
    Running = 1,
    Paused = 2
}

#[derive(PartialEq)]
pub enum TreasuryType {
    Open = 0,
    Locked = 1,
}