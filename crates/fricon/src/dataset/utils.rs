use std::{any::Any, sync::Arc};

use arrow_array::{Array, ArrayRef};

use crate::dataset::Error;

#[expect(clippy::needless_pass_by_value, reason = "Compatibility")]
pub fn downcast_array<T: Array + Any + Send + Sync + Clone>(
    array: ArrayRef,
) -> Result<Arc<T>, Error> {
    array
        .as_any()
        .downcast_ref::<T>()
        .map(|typed| Arc::new(typed.clone()))
        .ok_or(Error::IncompatibleType)
}
