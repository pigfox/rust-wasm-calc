//! Wasm-level tests for the #[wasm_bindgen]-exported surface.
//! Run with: wasm-pack test --node
//! These exercise the _js wrappers (exported as divide/sqrt/factorial),
//! get_history serialization, and the real wasm32 From<CalcError> for JsValue.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use wasm_calc::{factorial_js, CalcError, Calculator};

const EPSILON: f64 = 1e-10;

// ---- divide (divide_js) ----

#[wasm_bindgen_test]
fn divide_js_ok() {
    let mut c = Calculator::new();
    c.set_value(10.0);
    let v = c.divide_js(4.0).expect("divide should succeed");
    assert!((v - 2.5).abs() < EPSILON);
}

#[wasm_bindgen_test]
fn divide_js_by_zero_err_message() {
    let mut c = Calculator::new();
    c.set_value(1.0);
    let err = c.divide_js(0.0).expect_err("divide by zero must error");
    // Exercises the real wasm32 From<CalcError> for JsValue conversion
    assert_eq!(err.as_string().as_deref(), Some(CalcError::DivisionByZero.as_str()));
}

// ---- sqrt (sqrt_js) ----

#[wasm_bindgen_test]
fn sqrt_js_ok() {
    let mut c = Calculator::new();
    c.set_value(9.0);
    let v = c.sqrt_js().expect("sqrt should succeed");
    assert!((v - 3.0).abs() < EPSILON);
}

#[wasm_bindgen_test]
fn sqrt_js_negative_err_message() {
    let mut c = Calculator::new();
    c.set_value(-4.0);
    let err = c.sqrt_js().expect_err("negative sqrt must error");
    assert_eq!(err.as_string().as_deref(), Some(CalcError::NegativeSqrt.as_str()));
}

// ---- factorial (factorial_js) ----

#[wasm_bindgen_test]
fn factorial_js_ok() {
    let v = factorial_js(10.0).expect("factorial(10) should succeed");
    assert_eq!(v, 3_628_800.0);
}

#[wasm_bindgen_test]
fn factorial_js_overflow_err_message() {
    let err = factorial_js(21.0).expect_err("factorial(21) must overflow");
    assert_eq!(err.as_string().as_deref(), Some(CalcError::Overflow.as_str()));
}

// ---- get_history (serde_wasm_bindgen call site) ----

#[wasm_bindgen_test]
fn get_history_empty_is_array_not_null() {
    let c = Calculator::new();
    let h = c.get_history();
    assert_ne!(h, JsValue::NULL, "empty history must serialize to [], not null");
    assert!(js_sys::Array::is_array(&h));
    assert_eq!(js_sys::Array::from(&h).length(), 0);
}

#[wasm_bindgen_test]
fn get_history_after_ops_has_entries() {
    let mut c = Calculator::new();
    c.add(2.0);
    c.multiply(3.0);
    let _ = c.divide_js(2.0);
    let h = c.get_history();
    assert!(js_sys::Array::is_array(&h));
    let len = js_sys::Array::from(&h).length();
    assert!(len >= 3, "expected >=3 history entries, got {}", len);
    // Spot-check first entry is an object with expected shape
    let first = js_sys::Array::from(&h).get(0);
    assert!(first.is_object());
}

// ---- factorial ABI regression: ToUint32 wrap + non-integer inputs ----

#[wasm_bindgen_test]
fn factorial_js_rejects_u32_wrap_window() {
    // 2^32 + 5 previously truncated to 5 via ToUint32 and returned 120.
    // It is a well-formed integer, just far too large: the saturating cast
    // clamps it to u32::MAX so the core reports Overflow, not InvalidInput.
    let err = factorial_js(4294967301.0).expect_err("2^32+5 must be rejected");
    assert_eq!(err.as_string().as_deref(), Some(CalcError::Overflow.as_str()));
}

#[wasm_bindgen_test]
fn factorial_js_rejects_fractional_and_negative() {
    // Malformed inputs report InvalidInput, never a misleading "overflow".
    for bad in [3.5_f64, -1.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let err = factorial_js(bad).expect_err("malformed input must be rejected");
        assert_eq!(
            err.as_string().as_deref(),
            Some(CalcError::InvalidInput.as_str()),
            "wrong error for {}",
            bad
        );
    }
}

#[wasm_bindgen_test]
fn factorial_js_returns_number_semantics() {
    // Result is f64 now — exact for all n <= 20, including the max.
    let v = factorial_js(20.0).expect("factorial(20) should succeed");
    assert_eq!(v, 2_432_902_008_176_640_000.0);
}
