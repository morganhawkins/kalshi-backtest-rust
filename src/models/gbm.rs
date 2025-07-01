use statrs::function::erf::{erf, erf_inv};
use std::f64::consts::PI;

pub fn value(u_price: f64, strike: f64, sigma: f64, mu: f64, tte: f64) -> Option<f64> {
    if tte < 0.0 {
        return None;
    };

    let deviations_to_strike =
        f64::ln(strike / (u_price * tte * mu)) / (f64::sqrt(2.0 * tte) * sigma);
    let deriv_price = 0.5 * (1.0 - erf(deviations_to_strike));

    // checking that its finite. Must be valid price since must eval in (0,1)
    if deriv_price.is_finite() {
        return Some(deriv_price);
    } else {
        return None;
    }
}

pub fn iv(price: f64, u_price: f64, strike: f64, mu: f64, tte: f64) -> Option<f64> {
    // no solution to iv if contract market price is .5
    if price == 0.5 {
        return None;
    }

    let numerator = f64::ln(strike / (u_price * tte * mu));
    let denominator = f64::sqrt(2.0 * tte) * erf_inv(1.0 - (2.0 * price));

    let implied_vol = numerator / denominator;

    if implied_vol.is_finite() && (implied_vol >= 0.0) {
        return Some(implied_vol);
    } else {
        return None;
    }
}

pub fn delta(u_price: f64, strike: f64, sigma: f64, mu: f64, tte: f64) -> Option<f64> {
    let deviations_to_strike =
        (f64::ln(strike) - f64::ln(u_price) - (tte * mu)) / (f64::sqrt(2.0 * tte) * sigma);
    let denominator = u_price * sigma * f64::sqrt(2.0 * tte * PI);
    let delta_ = f64::exp(-deviations_to_strike.powi(2)) / denominator;

    if delta_.is_finite() {
        return Some(delta_);
    } else {
        return None;
    }
}

pub fn vega(u_price: f64, strike: f64, sigma: f64, mu: f64, tte: f64) -> Option<f64> {
    let expected_units_to_strike = f64::ln(strike) - f64::ln(u_price) - (tte * mu);
    let formula_lhs = f64::exp(-(expected_units_to_strike.powi(2)) / (2.0 * tte * (sigma.powi(2))));
    let vega_ =
        formula_lhs * ((expected_units_to_strike) / (sigma.powi(2) * f64::sqrt(2.0 * tte * PI)));

    if vega_.is_finite() {
        return Some(vega_);
    } else {
        return None;
    }
}

pub fn theta(u_price: f64, strike: f64, sigma: f64, mu: f64, tte: f64) -> Option<f64> {
    let formula_lhs = (f64::ln(u_price) - f64::ln(strike) - (tte * mu))
        / (sigma * f64::sqrt(8.0 * PI * tte.powi(3)));
    let theta_ = formula_lhs
        * f64::exp(
            -((f64::ln(strike) - f64::ln(u_price) - (tte * mu)).powi(2))
                / (2.0 * tte * (sigma.powi(2))),
        );

    if theta_.is_finite() {
        return Some(theta_);
    } else {
        return None;
    }
}

pub fn gamma(u_price: f64, strike: f64, sigma: f64, mu: f64, tte: f64) -> Option<f64> {
    let formula_lhs = f64::exp(
        -(f64::ln(strike) - f64::ln(u_price) - (tte * mu)).powi(2) / (2.0 * tte * sigma.powi(2)),
    );
    let gamma_ = formula_lhs
        * ((f64::ln(strike) - f64::ln(u_price) - (tte * (sigma.powi(2) + mu)))
            / (tte * (sigma.powi(3)) * (u_price.powi(2)) * f64::sqrt(2.0 * tte * PI)));

    if gamma_.is_finite() {
        return Some(gamma_);
    } else {
        return None;
    }
}
