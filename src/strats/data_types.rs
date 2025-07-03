use serde::Serialize;

#[derive(Serialize)]
pub struct TestResult {
    pub init_value: f64,
    pub deriv_term_value: f64,
    pub terminal_value: f64,
    pub strike: f64,
    pub start_ts: u64,
    pub expir_ts: u64,   
}