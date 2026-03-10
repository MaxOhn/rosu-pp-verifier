define_attrs! {
    pub struct OsuPerformanceAttributes (OsuRawPerformanceAttributes) {
        pub aim: f64,
        pub speed: f64,
        pub accuracy: f64,
        pub flashlight: f64,
        pub effective_miss_count: f64,
        pub speed_deviation: f64??,
        pub combo_based_estimated_miss_count: f64,
        pub score_based_estimated_miss_count: f64??,
        pub aim_estimated_slider_breaks: f64,
        pub speed_estimated_slider_breaks: f64,
        pub pp: f64,
    }
}

define_attrs! {
    pub struct TaikoPerformanceAttributes (TaikoRawPerformanceAttributes) {
        pub difficulty: f64,
        pub accuracy: f64,
        pub estimated_unstable_rate: f64??,
        pub pp: f64,
    }
}

define_attrs! {
    pub struct CatchPerformanceAttributes (CatchRawPerformanceAttributes) {
        pub pp: f64,
    }
}

define_attrs! {
    pub struct ManiaPerformanceAttributes (ManiaRawPerformanceAttributes) {
        pub difficulty: f64,
        pub pp: f64,
    }
}
