define_attrs! {
    pub struct OsuDifficultyAttributes (OsuRawDifficultyAttributes) {
        pub star_rating: f64,
        pub max_combo: u32,
        pub aim_difficulty: f64,
        pub aim_difficult_slider_count: f64,
        pub speed_difficulty: f64,
        pub speed_note_count: f64,
        pub slider_factor: f64,
        pub aim_top_weighted_slider_factor: f64,
        pub speed_top_weighted_slider_factor: f64,
        pub aim_difficult_strain_count: f64,
        pub speed_difficult_strain_count: f64,
        pub flashlight_difficulty: f64??,
        pub nested_score_per_object: f64,
        pub legacy_score_base_multiplier: f64,
        pub maximum_legacy_combo_score: f64,
    }
}

define_attrs! {
    pub struct TaikoDifficultyAttributes (TaikoRawDifficultyAttributes) {
        pub star_rating: f64,
        pub max_combo: u32,
        pub rhythm_difficulty: f64,
        pub mono_stamina_factor: f64,
        pub consistency_factor: f64,
    }
}

define_attrs! {
    pub struct CatchDifficultyAttributes (CatchRawDifficultyAttributes) {
        pub star_rating: f64,
        pub max_combo: u32,
    }
}

define_attrs! {
    pub struct ManiaDifficultyAttributes (ManiaRawDifficultyAttributes) {
        pub star_rating: f64,
        pub max_combo: u32,
    }
}
