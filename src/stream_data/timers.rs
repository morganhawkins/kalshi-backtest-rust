struct DeltaTimer{
    start_ts: u64,
    end_ts: u64,
    current_ts: RefCell<u64>,
    delta: u64,
}

impl DeltaTimer {
    pub fn new(start_ts: u64, end_ts: u64, current_ts: u64, delta: u64) -> Self{
        DeltaTimer{
            start_ts: start_ts, 
            end_ts: end_ts, 
            current_ts: RefCell::new(current_ts), 
            delta: delta}
    }

    pub fn cycle(&self) {
        let ts = self.current_ts.borrow_mut();
        ts += self.delta;
    }


}