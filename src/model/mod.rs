mod node;
mod predictor;
pub(super) mod utils;

pub mod macros {
    macro_rules! initialize {
        ($conn:expr) => {
            crate::model::CompressionTree::load($conn)
        };
        () => {
            crate::model::CompressionTree::new()
        };
    }

    pub(crate) use initialize;
}

// exports
pub(crate) use macros::initialize;
pub(crate) use predictor::CompressionTree;
