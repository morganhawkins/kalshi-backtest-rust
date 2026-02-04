/// Trait defining shared behavior for pricing models (e.g., GBM, Cauchy)
pub trait Model {
    /// Model-specific parameters for value and greeks (e.g., sigma/mu for GBM, gamma for Cauchy)
    type ModelInput;

    /// Compute theoretical value of the derivative
    fn value(params: &Self::ModelInput) -> Option<f64>;

    /// Compute implied volatility from market price
    fn iv(params: &Self::ModelInput) -> Option<f64>;

    /// Compute delta (sensitivity to underlying price)
    fn delta(params: &Self::ModelInput) -> Option<f64>;

    /// Compute vega (sensitivity to volatility parameter)
    fn vega(params: &Self::ModelInput) -> Option<f64>;

    /// Compute theta (sensitivity to time)
    fn theta(params: &Self::ModelInput) -> Option<f64>;

    /// Compute gamma (second derivative with respect to underlying price)
    fn gamma(params: &Self::ModelInput) -> Option<f64>;
}