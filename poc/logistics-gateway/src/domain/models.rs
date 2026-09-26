use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

impl Address {
    pub fn new(
        street: impl Into<String>,
        city: impl Into<String>,
        state: impl Into<String>,
        postal_code: impl Into<String>,
        country: impl Into<String>,
    ) -> Self {
        Self {
            street: street.into(),
            city: city.into(),
            state: state.into(),
            postal_code: postal_code.into(),
            country: country.into(),
        }
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.street.trim().is_empty() {
            return Err(ValidationError::EmptyField("street".to_string()));
        }
        if self.city.trim().is_empty() {
            return Err(ValidationError::EmptyField("city".to_string()));
        }
        if self.state.trim().is_empty() {
            return Err(ValidationError::EmptyField("state".to_string()));
        }
        if self.postal_code.trim().is_empty() {
            return Err(ValidationError::EmptyField("postal_code".to_string()));
        }
        if self.country.trim().is_empty() {
            return Err(ValidationError::EmptyField("country".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    pub length_cm: f64,
    pub width_cm: f64,
    pub height_cm: f64,
}

impl Dimensions {
    pub fn new(length_cm: f64, width_cm: f64, height_cm: f64) -> Result<Self, ValidationError> {
        if length_cm <= 0.0 || width_cm <= 0.0 || height_cm <= 0.0 {
            return Err(ValidationError::InvalidValue(
                "Dimensions must be positive".to_string(),
            ));
        }
        if length_cm > 300.0 || width_cm > 300.0 || height_cm > 300.0 {
            return Err(ValidationError::InvalidValue(
                "Dimensions exceed maximum size (300cm)".to_string(),
            ));
        }
        Ok(Self {
            length_cm,
            width_cm,
            height_cm,
        })
    }

    pub fn volumetric_weight_kg(&self) -> f64 {
        (self.length_cm * self.width_cm * self.height_cm) / 5000.0
    }

    pub fn billable_weight_kg(&self, actual_weight_kg: f64) -> f64 {
        actual_weight_kg.max(self.volumetric_weight_kg())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parcel {
    pub weight_kg: f64,
    pub dimensions: Dimensions,
}

impl Parcel {
    pub fn new(weight_kg: f64, dimensions: Dimensions) -> Result<Self, ValidationError> {
        if weight_kg <= 0.0 {
            return Err(ValidationError::InvalidValue(
                "Weight must be positive".to_string(),
            ));
        }
        if weight_kg > 50.0 {
            return Err(ValidationError::InvalidValue(
                "Weight exceeds maximum (50kg)".to_string(),
            ));
        }
        Ok(Self {
            weight_kg,
            dimensions,
        })
    }

    pub fn billable_weight_kg(&self) -> f64 {
        self.dimensions.billable_weight_kg(self.weight_kg)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub amount: rust_decimal::Decimal,
    pub currency: String,
}

impl Money {
    pub fn new(amount: rust_decimal::Decimal, currency: impl Into<String>) -> Self {
        Self {
            amount,
            currency: currency.into(),
        }
    }

    pub fn from_f64(amount: f64, currency: impl Into<String>) -> Self {
        Self {
            amount: amount.to_string().parse().unwrap_or_default(),
            currency: currency.into(),
        }
    }

    pub fn add(&self, other: &Money) -> Money {
        Money {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        }
    }

    pub fn multiply(&self, factor: f64) -> Money {
        Money {
            amount: self.amount
                * factor
                    .to_string()
                    .parse::<rust_decimal::Decimal>()
                    .unwrap_or_default(),
            currency: self.currency.clone(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Empty field: {0}")]
    EmptyField(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_validation() {
        let valid_address = Address::new("Street 123", "City", "State", "12345", "AR");
        assert!(valid_address.validate().is_ok());

        let invalid_address = Address::new("", "City", "State", "12345", "AR");
        assert!(invalid_address.validate().is_err());
    }

    #[test]
    fn test_dimensions_validation() {
        let valid_dims = Dimensions::new(30.0, 20.0, 15.0).unwrap();
        assert!(valid_dims.volumetric_weight_kg() > 0.0);

        let invalid_dims = Dimensions::new(-10.0, 20.0, 15.0);
        assert!(invalid_dims.is_err());
    }

    #[test]
    fn test_parcel_validation() {
        let dims = Dimensions::new(30.0, 20.0, 15.0).unwrap();
        let valid_parcel = Parcel::new(2.5, dims).unwrap();
        assert!(valid_parcel.billable_weight_kg() >= 2.5);

        let dims = Dimensions::new(30.0, 20.0, 15.0).unwrap();
        let invalid_parcel = Parcel::new(-1.0, dims);
        assert!(invalid_parcel.is_err());
    }

    #[test]
    fn test_money_operations() {
        let money1 = Money::from_f64(100.0, "ARS");
        let money2 = Money::from_f64(50.0, "ARS");
        let sum = money1.add(&money2);
        assert_eq!(sum.amount.to_string(), "150");

        let multiplied = money1.multiply(1.1);
        assert_eq!(multiplied.amount.to_string(), "110.0");
    }
}
