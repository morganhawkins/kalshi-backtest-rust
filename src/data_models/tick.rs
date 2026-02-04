/// Trait for types that represent a market update with a timestamp.
pub trait Tick {
    fn ts(&self) -> u64;
}
