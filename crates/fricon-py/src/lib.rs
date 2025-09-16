#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::significant_drop_tightening,
    clippy::needless_pass_by_value
)]

use pyo3::prelude::*;

mod cli;
mod conversion;
mod dataset;
mod trace;
mod workspace;
mod writer;

#[pymodule]
pub mod _core {
    #[pymodule_export]
    pub use crate::{
        cli::{main, main_gui, serve_workspace},
        dataset::Dataset,
        trace::Trace,
        workspace::{DatasetManager, Workspace},
        writer::DatasetWriter,
    };
}
