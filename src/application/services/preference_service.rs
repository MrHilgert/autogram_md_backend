use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use crate::application::ports::preference_repository::PreferenceRepository;
use crate::domain::user_preference::{InteractionSignal, ListingAttributes, UserPreference};

const DECAY: f64 = 0.95;
const PRUNE_THRESHOLD: f64 = 0.01;
const EMA_ALPHA: f64 = 0.3;

pub struct PreferenceService {
    repo: Arc<dyn PreferenceRepository>,
}

impl PreferenceService {
    pub fn new(repo: Arc<dyn PreferenceRepository>) -> Self {
        Self { repo }
    }

    pub async fn record_interaction(
        &self,
        user_id: Uuid,
        attrs: &ListingAttributes,
        signal: InteractionSignal,
    ) -> Result<(), anyhow::Error> {
        let mut pref = self
            .repo
            .get(user_id)
            .await?
            .unwrap_or_else(|| UserPreference::new(user_id));

        let w = signal.weight();

        // 1. Decay all existing weights
        decay_map(&mut pref.make_weights);
        decay_map(&mut pref.model_weights);
        decay_map(&mut pref.body_weights);
        decay_map(&mut pref.fuel_weights);
        decay_map(&mut pref.trans_weights);
        decay_map(&mut pref.drive_weights);

        // 2. Add signal weight
        add_weight(&mut pref.make_weights, &attrs.make_id.to_string(), w);
        add_weight(&mut pref.model_weights, &attrs.model_id.to_string(), w);
        add_weight(&mut pref.body_weights, &attrs.body, w);
        add_weight(&mut pref.fuel_weights, &attrs.fuel, w);
        add_weight(&mut pref.trans_weights, &attrs.transmission, w);
        if let Some(drive) = &attrs.drive {
            add_weight(&mut pref.drive_weights, drive, w);
        }

        // 3. Update price/year centers (EMA)
        pref.price_center = Some(match pref.price_center {
            Some(center) => center * (1.0 - EMA_ALPHA) + (attrs.price as f64) * EMA_ALPHA,
            None => attrs.price as f64,
        });
        pref.year_center = Some(match pref.year_center {
            Some(center) => center * (1.0 - EMA_ALPHA) + (attrs.year as f64) * EMA_ALPHA,
            None => attrs.year as f64,
        });

        // 4. Increment total interactions (positive signals only)
        if signal.is_positive() {
            pref.total_interactions += 1;
        }

        // 5. Prune negligible weights
        prune_map(&mut pref.make_weights);
        prune_map(&mut pref.model_weights);
        prune_map(&mut pref.body_weights);
        prune_map(&mut pref.fuel_weights);
        prune_map(&mut pref.trans_weights);
        prune_map(&mut pref.drive_weights);

        pref.updated_at = chrono::Utc::now();

        self.repo.upsert(&pref).await?;
        Ok(())
    }
}

fn decay_map(map: &mut HashMap<String, f64>) {
    for v in map.values_mut() {
        *v *= DECAY;
    }
}

fn add_weight(map: &mut HashMap<String, f64>, key: &str, weight: f64) {
    let entry = map.entry(key.to_string()).or_insert(0.0);
    *entry = (*entry + weight).max(0.0);
}

fn prune_map(map: &mut HashMap<String, f64>) {
    map.retain(|_, v| *v >= PRUNE_THRESHOLD);
}
