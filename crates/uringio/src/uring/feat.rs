use crate::{
    platform::iouring::IoUringFeatureFlags,
    shared::{
        error::{Result, err},
        null::{NULL, Null},
    },
};

pub fn check_setup_features(features: IoUringFeatureFlags) -> Result<Null> {
    if !features.contains(IoUringFeatureFlags::SINGLE_MMAP) {
        return err!("Feature IORING_FEAT_SINGLE_MMAP Not Supported");
    }

    if !features.contains(IoUringFeatureFlags::NODROP) {
        return err!("Feature IORING_FEAT_NODROP Not Supported");
    }

    Ok(NULL)
}
