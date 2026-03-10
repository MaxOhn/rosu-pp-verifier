define_attrs! {
    pub struct OsuDifficultyAttributes (OsuRawDifficultyAttributes) {
        pub star_rating: f64,
        pub max_combo: u32,
        pub aim_difficulty: f64,
        pub aim_difficult_slider_count: f64,
        pub speed_difficulty: f64,
        pub speed_note_count: f64,
        pub aim_difficult_strain_count: f64,
        pub speed_difficult_strain_count: f64,
        pub flashlight_difficulty: f64??,
        pub slider_factor: f64,
    }
}

define_attrs! {
    pub struct TaikoDifficultyAttributes (TaikoRawDifficultyAttributes) {
        pub star_rating: f64,
        pub max_combo: u32,
        pub stamina_difficulty: f64,
        pub mono_stamina_factor: f64,
        pub rhythm_difficulty: f64,
        pub colour_difficulty: f64,
        pub reading_difficulty: f64,

        // Unused but required to be defined to not trigger
        // `deny_unknown_fields` error
        pub rhythm_difficult_strains: f64,
        pub colour_difficult_strains: f64,
        pub stamina_difficult_strains: f64,
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
