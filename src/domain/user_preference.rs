use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    pub user_id: Uuid,
    pub make_weights: HashMap<String, f64>,
    pub model_weights: HashMap<String, f64>,
    pub body_weights: HashMap<String, f64>,
    pub fuel_weights: HashMap<String, f64>,
    pub trans_weights: HashMap<String, f64>,
    pub drive_weights: HashMap<String, f64>,
    pub price_center: Option<f64>,
    pub year_center: Option<f64>,
    pub total_interactions: i32,
    pub updated_at: DateTime<Utc>,
}

impl UserPreference {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            make_weights: HashMap::new(),
            model_weights: HashMap::new(),
            body_weights: HashMap::new(),
            fuel_weights: HashMap::new(),
            trans_weights: HashMap::new(),
            drive_weights: HashMap::new(),
            price_center: None,
            year_center: None,
            total_interactions: 0,
            updated_at: Utc::now(),
        }
    }

    pub fn is_cold_start(&self) -> bool {
        self.total_interactions < 5
    }
}

pub enum InteractionSignal {
    View,
    Like,
    Favorite,
    Unlike,
    Unfavorite,
}

impl InteractionSignal {
    pub fn weight(&self) -> f64 {
        match self {
            Self::View => 1.0,
            Self::Like => 3.0,
            Self::Favorite => 5.0,
            Self::Unlike => -3.0,
            Self::Unfavorite => -5.0,
        }
    }

    pub fn is_positive(&self) -> bool {
        self.weight() > 0.0
    }
}

/// Lightweight listing attributes needed for preference updates.
pub struct ListingAttributes {
    pub make_id: i32,
    pub model_id: i32,
    pub body: String,
    pub fuel: String,
    pub transmission: String,
    pub drive: Option<String>,
    pub price: i32,
    pub year: i16,
}
