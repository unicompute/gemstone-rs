use gemstone_rs::py_native::{
    capabilities, samples_report, smoke_dry_run_report, PyNativeErrorInfo, PyNativeSession,
    PyNativeValue,
};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use std::cell::RefCell;

#[pyfunction]
fn capabilities_json() -> String {
    capabilities().to_json()
}

#[pyfunction]
fn smoke_dry_run_json() -> String {
    smoke_dry_run_report().to_json()
}

#[pyfunction]
fn samples_json() -> String {
    samples_report().to_json()
}

#[pyclass(unsendable)]
struct NativeSession {
    inner: RefCell<Option<PyNativeSession>>,
}

#[pymethods]
impl NativeSession {
    #[staticmethod]
    fn login_from_env() -> PyResult<Self> {
        Ok(Self {
            inner: RefCell::new(Some(
                PyNativeSession::login_from_env().map_err(py_native_error)?,
            )),
        })
    }

    fn session_id(&self) -> PyResult<i32> {
        with_session(&self.inner, |session| Ok(session.session_id()))
    }

    fn eval_repr(&self, source: &str) -> PyResult<String> {
        with_session(&self.inner, |session| {
            session
                .eval(source)
                .map(|value| format!("{value:?}"))
                .map_err(py_native_error)
        })
    }

    fn eval_smallint(&self, source: &str) -> PyResult<i64> {
        with_session(&self.inner, |session| match session.eval(source) {
            Ok(PyNativeValue::SmallInt(value)) => Ok(value),
            Ok(other) => Err(PyValueError::new_err(format!(
                "expected SmallInt from eval, got {other:?}"
            ))),
            Err(error) => Err(py_native_error(error)),
        })
    }

    fn logout(&self) -> PyResult<()> {
        if let Some(mut session) = self.inner.borrow_mut().take() {
            session.logout().map_err(py_native_error)?;
        }
        Ok(())
    }
}

impl Drop for NativeSession {
    fn drop(&mut self) {
        if let Some(mut session) = self.inner.borrow_mut().take() {
            let _ = session.logout();
        }
    }
}

fn with_session<T>(
    inner: &RefCell<Option<PyNativeSession>>,
    f: impl FnOnce(&mut PyNativeSession) -> PyResult<T>,
) -> PyResult<T> {
    let mut guard = inner.borrow_mut();
    let session = guard
        .as_mut()
        .ok_or_else(|| PyRuntimeError::new_err("GemStone session is logged out"))?;
    f(session)
}

fn py_native_error(error: gemstone_rs::Error) -> PyErr {
    let info = PyNativeErrorInfo::from_error(&error);
    PyRuntimeError::new_err(format!("{:?}: {}", info.kind, info.message))
}

#[pymodule]
fn gemstone_py_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(capabilities_json, m)?)?;
    m.add_function(wrap_pyfunction!(samples_json, m)?)?;
    m.add_function(wrap_pyfunction!(smoke_dry_run_json, m)?)?;
    m.add_class::<NativeSession>()?;
    Ok(())
}
