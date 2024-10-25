mod predictor;
pub(super) mod utils;

pub mod macros {
    macro_rules! initialize {
        ($a:literal) => {
            crate::model::BlockSizePredictor::initialize($a.to_string())
        };
        ($a:expr) => {
            crate::model::BlockSizePredictor::initialize($a)
        };
        () => {
            crate::model::BlockSizePredictor::initialize("model.json".to_string())
        };
    }

    pub(crate) use initialize;
}

// exports
pub(crate) use macros::initialize;
pub(crate) use predictor::BlockSizePredictor;
