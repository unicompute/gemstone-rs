use gemstone_rs::py_native::{
    capabilities, migration_report, samples_report, smoke_dry_run_report, PyNativeErrorInfo,
    PyNativeSession, PyNativeValue,
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

#[pyfunction]
fn migration_json() -> String {
    migration_report().to_json()
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

    fn eval_oop(&self, source: &str) -> PyResult<u64> {
        with_session(&self.inner, |session| {
            session.eval_oop(source).map_err(py_native_error)
        })
    }

    fn execute(&self, source: &str) -> PyResult<u64> {
        with_session(&self.inner, |session| {
            session.execute(source).map_err(py_native_error)
        })
    }

    fn resolve(&self, name: &str) -> PyResult<u64> {
        with_session(&self.inner, |session| {
            session.resolve(name).map_err(py_native_error)
        })
    }

    fn perform_raw_oop(&self, receiver: u64, selector: &str, args: Vec<u64>) -> PyResult<u64> {
        with_session(&self.inner, |session| {
            session
                .perform_oop_raw(receiver, selector, &args)
                .map_err(py_native_error)
        })
    }

    fn new_string(&self, value: &str) -> PyResult<u64> {
        with_session(&self.inner, |session| {
            session.new_string(value).map_err(py_native_error)
        })
    }

    fn fetch_string(&self, oop: u64) -> PyResult<String> {
        with_session(&self.inner, |session| {
            session.fetch_string(oop).map_err(py_native_error)
        })
    }

    fn global_get(&self, symbol_name: &str) -> PyResult<u64> {
        with_session(&self.inner, |session| {
            session.global_get(symbol_name).map_err(py_native_error)
        })
    }

    fn global_put_raw(&self, symbol_name: &str, value: u64) -> PyResult<()> {
        with_session(&self.inner, |session| {
            session
                .global_put_raw(symbol_name, value)
                .map_err(py_native_error)
        })
    }

    fn global_put_string(&self, symbol_name: &str, value: &str) -> PyResult<()> {
        with_session(&self.inner, |session| {
            session
                .global_put_value(symbol_name, PyNativeValue::String(value.to_string()))
                .map_err(py_native_error)
        })
    }

    fn global_put_smallint(&self, symbol_name: &str, value: i64) -> PyResult<()> {
        with_session(&self.inner, |session| {
            session
                .global_put_value(symbol_name, PyNativeValue::SmallInt(value))
                .map_err(py_native_error)
        })
    }

    fn add_to_export_set(&self, oop: u64) -> PyResult<()> {
        with_session(&self.inner, |session| {
            session.add_to_export_set(oop).map_err(py_native_error)
        })
    }

    fn remove_from_export_set(&self, oop: u64) -> PyResult<()> {
        with_session(&self.inner, |session| {
            session.remove_from_export_set(oop).map_err(py_native_error)
        })
    }

    fn needs_commit(&self) -> PyResult<bool> {
        with_session(&self.inner, |session| {
            session.needs_commit().map_err(py_native_error)
        })
    }

    fn in_transaction(&self) -> PyResult<bool> {
        with_session(&self.inner, |session| {
            session.in_transaction().map_err(py_native_error)
        })
    }

    fn commit(&self) -> PyResult<()> {
        with_session(&self.inner, |session| {
            session.commit().map_err(py_native_error)
        })
    }

    fn abort(&self) -> PyResult<()> {
        with_session(&self.inner, |session| {
            session.abort().map_err(py_native_error)
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
    m.add_function(wrap_pyfunction!(migration_json, m)?)?;
    m.add_function(wrap_pyfunction!(samples_json, m)?)?;
    m.add_function(wrap_pyfunction!(smoke_dry_run_json, m)?)?;
    m.add_class::<NativeSession>()?;
    Ok(())
}
