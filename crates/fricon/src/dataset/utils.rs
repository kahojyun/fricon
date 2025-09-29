use std::{any::Any, sync::Arc};

use arrow_array::{Array, ArrayRef};

use crate::dataset::Error;

pub fn downcast_array<T: Array + Any + Send + Sync>(array: ArrayRef) -> Result<Arc<T>, Error> {
    if array.as_any().is::<T>() {
        let raw = Arc::into_raw(array);
        let ptr = raw.cast();
        // SAFETY: Type checked
        Ok(unsafe { Arc::from_raw(ptr) })
    } else {
        Err(Error::IncompatibleType)
    }
}
