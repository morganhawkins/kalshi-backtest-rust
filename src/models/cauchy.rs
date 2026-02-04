use std::f64::consts::PI;

use super::model::Model;

pub struct CauchyInput {
    pub u_price: f64,
    pub strike: f64,
    pub gamma: f64,
    pub tte: f64,
    pub price: Option<f64>, // market price, needed for iv calculation
}

pub struct Cauchy;

impl Model for Cauchy {
    type ModelInput = CauchyInput;

    /// Computes the theoretical value using the Cauchy distribution CDF.
    /// Returns the probability that the underlying exceeds the strike at expiration.
    fn value(params: &Self::ModelInput) -> Option<f64> {
        value(params.u_price, params.strike, params.gamma, params.tte)
    }

    /// Computes implied volatility (gamma parameter) from market price.
    /// Uses closed-form inversion of the Cauchy pricing formula.
    /// Returns `None` if price equals 0.5 (no unique solution).
    fn iv(params: &Self::ModelInput) -> Option<f64> {
        let price = params.price?;
        iv(price, params.u_price, params.strike, params.tte)
    }

    /// Computes delta as the derivative of value with respect to underlying price.
    /// Uses the Cauchy PDF in the chain rule calculation.
    fn delta(params: &Self::ModelInput) -> Option<f64> {
        delta(params.u_price, params.strike, params.gamma, params.tte)
    }

    /// Computes vega as the derivative of value with respect to gamma.
    /// Measures sensitivity to changes in the Cauchy scale parameter.
    fn vega(params: &Self::ModelInput) -> Option<f64> {
        vega(params.u_price, params.strike, params.gamma, params.tte)
    }

    /// Computes theta as the derivative of value with respect to time.
    /// Represents time decay of the option value.
    fn theta(params: &Self::ModelInput) -> Option<f64> {
        theta(params.u_price, params.strike, params.gamma, params.tte)
    }

    /// Computes gamma as the second derivative of value with respect to underlying price.
    /// Measures the rate of change of delta.
    fn gamma(params: &Self::ModelInput) -> Option<f64> {
        gamma(params.u_price, params.strike, params.gamma, params.tte)
    }
}

// cdf of cauchy distribution, no option for non-central
pub(crate) fn cauch_cdf(x: f64, gamma: f64) -> f64{
    (f64::atan(x/gamma)/PI) + 0.5
}

// pdf of cauchy distribution, no option for non-central
pub(crate) fn cauch_pdf(x: f64, gamma: f64) -> f64 {
    let denom = ((x/gamma).powi(2) + 1.0) * gamma * PI;
    denom.recip()
}

pub(crate) fn value(u_price: f64, strike: f64, gamma:f64, tte: f64) -> Option<f64> {
    let inner = f64::ln(strike/u_price) / (tte*gamma);
    let value_ = 1.0 - cauch_cdf(inner, gamma);
    
    // check for nan
    if value_.is_finite(){
        Some(value_)
    } else {
        None
    }
}

pub(crate) fn iv(price: f64, u_price: f64, strike: f64, tte: f64) -> Option<f64> {
    // no solution to iv if contract market price is .5
    if price == 0.5 {
        return None;
    }

    let numerator = f64::ln(strike/u_price);
    let denominator = tte * f64::tan(PI*(0.5-price));
    let iv_ = f64::sqrt(numerator / denominator);

    // check for nan 
    if iv_.is_finite() {
        Some(iv_)
    } else {
        None
    }

}

pub(crate) fn delta(u_price: f64, strike: f64, gamma: f64, tte: f64) -> Option<f64> {
    let inner = f64::ln(strike/u_price) / (tte*gamma);
    let delta_ = cauch_pdf(inner, gamma) / (tte*gamma*u_price);

    // check for nan
    if delta_.is_finite() {
        Some(delta_)
    } else {
        None
    }
}

pub(crate) fn vega(u_price: f64, strike: f64, gamma: f64, tte: f64) -> Option<f64> {
    let inner = f64::ln(strike/u_price) / (tte*gamma);
    let vega_ = cauch_pdf(inner, gamma) * (inner/gamma);
    
    // check for nan
    if vega_.is_finite() {
        return Some(vega_);
    } else {
        return None;
    }
}

pub(crate) fn theta(u_price: f64, strike: f64, gamma: f64, tte: f64) -> Option<f64> {
    let inner = f64::ln(strike/u_price) / (tte*gamma);
    let theta_ = -cauch_pdf(inner, gamma) * (inner/tte);

    // check for nan
    if theta_.is_finite() {
        return Some(theta_);
    } else {
        return None;
    }
}

pub(crate) fn gamma(u_price: f64, strike: f64, gamma: f64, tte: f64) -> Option<f64> {
    let numer = 2.0 * f64::ln(strike/u_price);
    let denom_lhs = PI * gamma.powi(2) * tte.powi(3) * u_price.powi(2);
    let denom_rhs = (f64::ln(strike/u_price) / (tte*gamma)).powi(2) + gamma.powi(2);
    let gamma_ = numer / (denom_lhs * denom_rhs.powi(2));
    
    // check for nan
    if gamma_.is_finite() {
        return Some(gamma_);
    } else {
        return None;
    }
}

// TODO: check that these tests also make sense for cauchy process
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        // test values are on right side of .5
        assert!(value(10.0, 11.0, 1.0, 1.0).unwrap() < 0.5);
        assert!(value(12.0, 11.0, 1.0, 1.0).unwrap() > 0.5);
    }

    #[test]
    fn test_iv() {
        assert!(iv(0.5, 10.0, 10.0, 1.0).is_none());
        assert!(iv(0.7, 10.0, 12.0, 1.0).is_none());
        assert!(iv(0.3, 10.0, 12.0, 1.0).unwrap() > 0.0);
    }
    #[test]
    fn test_delta() {
        assert!(delta(10.0, 15.0, 1.0, 1.0).unwrap() > 0.0);
        assert!(delta(15.0, 10.0, 1.0, 1.0).unwrap() > 0.0);
    }
    #[test]
    fn test_vega() {
        assert!(vega(10.0, 15.0, 0.1, 1.0).unwrap() > 0.0);
        assert!(vega(15.0, 10.0, 0.1, 1.0).unwrap() < 0.0);
    }
    #[test]
    fn test_gamma() {
    }
    #[test]
    fn test_theta() {
        assert!(theta(10.0, 15.0, 0.1, 1.0).unwrap() < 0.0);
        assert!(theta(15.0, 10.0, 0.1, 1.0).unwrap() > 0.0);
    }
}
