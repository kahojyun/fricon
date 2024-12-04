use pyo3::prelude::*;

#[pyfunction]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

#[pymodule]
pub mod _core {
    #[pymodule_export]
    pub use super::add;
}
