use crate::{
    platform::iouring::IoUringFeatureFlags,
    shared::{
        error::{Result, err},
        null::{NULL, Null},
    },
};

pub fn check_setup_features(features: IoUringFeatureFlags) -> Result<Null> {
    if !features.contains(IoUringFeatureFlags::NODROP) {
        return err!("Feature IORING_FEAT_NODROP Invalid");
    }

    Ok(NULL)
}
