macro_rules! define_attrs {
    (
        $( #[ $meta:meta ] )*
        pub struct $name:ident ($raw_name:ident) {
            $(
                $( #[serde($serde_meta:meta)] )?
                $vis:vis $field:ident: $ty:ident $( ? $option:tt )?,
            )*
        }
    ) => {
        #[derive(Debug, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $raw_name {
            $(
                $( #[serde($serde_meta)] )?
                $field: define_attrs!(@RAW_FIELD $ty $( $option )?),
            )*
        }

        $( #[ $meta ] )*
        #[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
        #[rkyv(attr(derive(Debug)))]
        pub struct $name {
            $( $vis $field: define_attrs!(@FIELD $ty $( $option )?), )*
        }

        impl From<$raw_name> for $name {
            fn from(attrs: $raw_name) -> Self {
                Self {
                    $( $field: define_attrs!(@FIELD_INTO attrs $field: $ty $( $option )?) , )*
                }
            }
        }
    };
    ( @FIELD f64?) => {
        Option<f64>
    };
    ( @FIELD $ty:ty) => {
        $ty
    };
    ( @RAW_FIELD f64) => {
        serde_json::Number
    };
    ( @RAW_FIELD f64?) => {
        Option<serde_json::Number>
    };
    ( @RAW_FIELD $ty:ty) => {
        $ty
    };
    ( @FIELD_INTO $attrs:ident $field:ident: f64) => {
        $attrs.$field.as_f64().unwrap().into()
    };
    ( @FIELD_INTO $attrs:ident $field:ident: f64?) => {
        $attrs.$field.as_ref().and_then(serde_json::Number::as_f64)
    };
    ( @FIELD_INTO $attrs:ident $field:ident: $ty:ty) => {{
        $attrs.$field
    }};
}

pub mod data_score;
pub mod difficulty;
pub mod mode;
pub mod mods;
pub mod object;
pub mod performance;
pub mod recent_map;
pub mod simulate_score;
pub mod statistics;
