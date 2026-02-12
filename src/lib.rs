use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    DivisionByZero,
    NegativeSqrt,
    Overflow,
}

impl CalcError {
    fn as_str(&self) -> &str {
        match self {
            CalcError::DivisionByZero => "Division by zero",
            CalcError::NegativeSqrt => "Cannot take square root of negative number",
            CalcError::Overflow => "Factorial overflow: n must be <= 20",
        }
    }
}

// Always implement From for compilation, but only actually use it in WASM
impl From<CalcError> for JsValue {
    fn from(err: CalcError) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            JsValue::from_str(err.as_str())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Fallback for non-WASM (shouldn't be called)
            JsValue::NULL
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationHistory {
    operand1: f64,
    operand2: f64,
    operation: Operation,
    result: f64,
}

#[wasm_bindgen]
pub struct Calculator {
    current_value: f64,
    memory: f64,
    history: Vec<CalculationHistory>,
}

// Core implementation without WASM bindings (for tests)
impl Calculator {
    pub fn divide(&mut self, value: f64) -> Result<f64, CalcError> {
        if value == 0.0 {
            return Err(CalcError::DivisionByZero);
        }
        let result = self.current_value / value;
        self.add_to_history(self.current_value, value, Operation::Divide, result);
        self.current_value = result;
        Ok(result)
    }

    pub fn sqrt(&mut self) -> Result<f64, CalcError> {
        if self.current_value < 0.0 {
            return Err(CalcError::NegativeSqrt);
        }
        self.current_value = self.current_value.sqrt();
        Ok(self.current_value)
    }
}

// WASM bindings for JavaScript
#[wasm_bindgen]
impl Calculator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Calculator {
        Calculator {
            current_value: 0.0,
            memory: 0.0,
            history: Vec::new(),
        }
    }

    pub fn add(&mut self, value: f64) -> f64 {
        let result = self.current_value + value;
        self.add_to_history(self.current_value, value, Operation::Add, result);
        self.current_value = result;
        result
    }

    pub fn subtract(&mut self, value: f64) -> f64 {
        let result = self.current_value - value;
        self.add_to_history(self.current_value, value, Operation::Subtract, result);
        self.current_value = result;
        result
    }

    pub fn multiply(&mut self, value: f64) -> f64 {
        let result = self.current_value * value;
        self.add_to_history(self.current_value, value, Operation::Multiply, result);
        self.current_value = result;
        result
    }

    #[wasm_bindgen(js_name = divide)]
    pub fn divide_js(&mut self, value: f64) -> Result<f64, JsValue> {
        self.divide(value).map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = sqrt)]
    pub fn sqrt_js(&mut self) -> Result<f64, JsValue> {
        self.sqrt().map_err(|e| e.into())
    }

    pub fn power(&mut self, exponent: f64) -> f64 {
        self.current_value = self.current_value.powf(exponent);
        self.current_value
    }

    pub fn get_value(&self) -> f64 {
        self.current_value
    }

    pub fn set_value(&mut self, value: f64) {
        self.current_value = value;
    }

    pub fn clear(&mut self) {
        self.current_value = 0.0;
    }

    pub fn memory_store(&mut self) {
        self.memory = self.current_value;
    }

    pub fn memory_recall(&mut self) -> f64 {
        self.current_value = self.memory;
        self.memory
    }

    pub fn memory_clear(&mut self) {
        self.memory = 0.0;
    }

    pub fn memory_add(&mut self) {
        self.memory += self.current_value;
    }

    pub fn get_memory(&self) -> f64 {
        self.memory
    }

    pub fn get_history(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.history).unwrap_or(JsValue::NULL)
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn history_count(&self) -> usize {
        self.history.len()
    }
}

impl Calculator {
    fn add_to_history(&mut self, operand1: f64, operand2: f64, operation: Operation, result: f64) {
        self.history.push(CalculationHistory {
            operand1,
            operand2,
            operation,
            result,
        });
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Calculator {
            current_value: 0.0,
            memory: 0.0,
            history: Vec::new(),
        }
    }
}

// Standalone utility functions
#[wasm_bindgen]
pub fn percentage(value: f64, percent: f64) -> f64 {
    value * (percent / 100.0)
}

#[wasm_bindgen]
pub fn compound_interest(principal: f64, rate: f64, years: f64, compounds_per_year: f64) -> f64 {
    principal * (1.0 + rate / (100.0 * compounds_per_year)).powf(compounds_per_year * years)
}

// Core factorial implementation (for tests)
pub fn factorial(n: u32) -> Result<u64, CalcError> {
    if n > 20 {
        return Err(CalcError::Overflow);
    }
    
    let mut result: u64 = 1;
    for i in 2..=n {
        result = result.checked_mul(i as u64)
            .ok_or(CalcError::Overflow)?;
    }
    Ok(result)
}

// WASM wrapper for factorial
#[wasm_bindgen]
pub fn factorial_js(n: u32) -> Result<u64, JsValue> {
    factorial(n).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_calculator() {
        let calc = Calculator::new();
        assert_eq!(calc.get_value(), 0.0);
        assert_eq!(calc.get_memory(), 0.0);
        assert_eq!(calc.history_count(), 0);
    }

    #[test]
    fn test_add() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        let result = calc.add(5.0);
        assert_eq!(result, 15.0);
        assert_eq!(calc.get_value(), 15.0);
    }

    #[test]
    fn test_add_negative() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        let result = calc.add(-5.0);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_subtract() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        let result = calc.subtract(3.0);
        assert_eq!(result, 7.0);
        assert_eq!(calc.get_value(), 7.0);
    }

    #[test]
    fn test_subtract_negative_result() {
        let mut calc = Calculator::new();
        calc.set_value(5.0);
        let result = calc.subtract(10.0);
        assert_eq!(result, -5.0);
    }

    #[test]
    fn test_multiply() {
        let mut calc = Calculator::new();
        calc.set_value(4.0);
        let result = calc.multiply(3.0);
        assert_eq!(result, 12.0);
        assert_eq!(calc.get_value(), 12.0);
    }

    #[test]
    fn test_multiply_by_zero() {
        let mut calc = Calculator::new();
        calc.set_value(5.0);
        let result = calc.multiply(0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_multiply_negative() {
        let mut calc = Calculator::new();
        calc.set_value(-4.0);
        let result = calc.multiply(3.0);
        assert_eq!(result, -12.0);
    }

    #[test]
    fn test_divide() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        let result = calc.divide(2.0).unwrap();
        assert_eq!(result, 5.0);
        assert_eq!(calc.get_value(), 5.0);
    }

    #[test]
    fn test_divide_by_zero() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        let result = calc.divide(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_divide_negative() {
        let mut calc = Calculator::new();
        calc.set_value(-10.0);
        let result = calc.divide(2.0).unwrap();
        assert_eq!(result, -5.0);
    }

    #[test]
    fn test_sqrt() {
        let mut calc = Calculator::new();
        calc.set_value(16.0);
        let result = calc.sqrt().unwrap();
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_sqrt_negative() {
        let mut calc = Calculator::new();
        calc.set_value(-4.0);
        let result = calc.sqrt();
        assert!(result.is_err());
    }

    #[test]
    fn test_sqrt_zero() {
        let mut calc = Calculator::new();
        calc.set_value(0.0);
        let result = calc.sqrt().unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_power() {
        let mut calc = Calculator::new();
        calc.set_value(2.0);
        let result = calc.power(3.0);
        assert_eq!(result, 8.0);
    }

    #[test]
    fn test_power_zero() {
        let mut calc = Calculator::new();
        calc.set_value(5.0);
        let result = calc.power(0.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_power_negative() {
        let mut calc = Calculator::new();
        calc.set_value(2.0);
        let result = calc.power(-2.0);
        assert_eq!(result, 0.25);
    }

    #[test]
    fn test_clear() {
        let mut calc = Calculator::new();
        calc.set_value(42.0);
        calc.clear();
        assert_eq!(calc.get_value(), 0.0);
    }

    #[test]
    fn test_memory_store() {
        let mut calc = Calculator::new();
        calc.set_value(15.0);
        calc.memory_store();
        assert_eq!(calc.get_memory(), 15.0);
    }

    #[test]
    fn test_memory_recall() {
        let mut calc = Calculator::new();
        calc.set_value(20.0);
        calc.memory_store();
        calc.set_value(5.0);
        let recalled = calc.memory_recall();
        assert_eq!(recalled, 20.0);
        assert_eq!(calc.get_value(), 20.0);
    }

    #[test]
    fn test_memory_clear() {
        let mut calc = Calculator::new();
        calc.set_value(30.0);
        calc.memory_store();
        calc.memory_clear();
        assert_eq!(calc.get_memory(), 0.0);
    }

    #[test]
    fn test_memory_add() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        calc.memory_store();
        calc.set_value(5.0);
        calc.memory_add();
        assert_eq!(calc.get_memory(), 15.0);
    }

    #[test]
    fn test_memory_add_multiple_times() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        calc.memory_add();
        calc.set_value(5.0);
        calc.memory_add();
        calc.set_value(3.0);
        calc.memory_add();
        assert_eq!(calc.get_memory(), 18.0);
    }

    #[test]
    fn test_history_tracking() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        calc.add(5.0);
        calc.multiply(2.0);
        
        assert_eq!(calc.history_count(), 2);
    }

    #[test]
    fn test_clear_history() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        calc.add(5.0);
        calc.clear_history();
        
        assert_eq!(calc.history_count(), 0);
    }

    #[test]
    fn test_chained_operations() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        calc.add(5.0);      // 15
        calc.multiply(2.0);  // 30
        calc.subtract(10.0); // 20
        calc.divide(4.0).unwrap(); // 5
        
        assert_eq!(calc.get_value(), 5.0);
        assert_eq!(calc.history_count(), 4);
    }

    #[test]
    fn test_percentage() {
        assert_eq!(percentage(200.0, 10.0), 20.0);
        assert_eq!(percentage(50.0, 50.0), 25.0);
        assert_eq!(percentage(100.0, 0.0), 0.0);
    }

    #[test]
    fn test_percentage_over_100() {
        assert_eq!(percentage(50.0, 200.0), 100.0);
    }

    #[test]
    fn test_compound_interest() {
        let result = compound_interest(1000.0, 5.0, 10.0, 12.0);
        // Expected: 1000 * (1 + 0.05/12)^(12*10)
        assert!((result - 1647.01).abs() < 1.0);
    }

    #[test]
    fn test_compound_interest_yearly() {
        let result = compound_interest(1000.0, 10.0, 5.0, 1.0);
        // 1000 * 1.1^5
        assert!((result - 1610.51).abs() < 1.0);
    }

    #[test]
    fn test_factorial_zero() {
        assert_eq!(factorial(0).unwrap(), 1);
    }

    #[test]
    fn test_factorial_one() {
        assert_eq!(factorial(1).unwrap(), 1);
    }

    #[test]
    fn test_factorial_five() {
        assert_eq!(factorial(5).unwrap(), 120);
    }

    #[test]
    fn test_factorial_ten() {
        assert_eq!(factorial(10).unwrap(), 3628800);
    }

    #[test]
    fn test_factorial_overflow() {
        let result = factorial(21);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_trait() {
        let calc = Calculator::default();
        assert_eq!(calc.get_value(), 0.0);
        assert_eq!(calc.get_memory(), 0.0);
    }

    #[test]
    fn test_floating_point_precision() {
        let mut calc = Calculator::new();
        calc.set_value(0.1);
        calc.add(0.2);
        // Account for floating point imprecision
        assert!((calc.get_value() - 0.3).abs() < 0.0000001);
    }

    #[test]
    fn test_large_numbers() {
        let mut calc = Calculator::new();
        calc.set_value(1e100);
        calc.multiply(2.0);
        assert_eq!(calc.get_value(), 2e100);
    }

    #[test]
    fn test_very_small_numbers() {
        let mut calc = Calculator::new();
        calc.set_value(1e-100);
        calc.multiply(2.0);
        assert_eq!(calc.get_value(), 2e-100);
    }

    #[test]
    fn test_set_value() {
        let mut calc = Calculator::new();
        calc.set_value(42.0);
        assert_eq!(calc.get_value(), 42.0);
    }

    #[test]
    fn test_operation_enum_equality() {
        assert_eq!(Operation::Add, Operation::Add);
        assert_ne!(Operation::Add, Operation::Subtract);
    }

    #[test]
    fn test_operation_enum_clone() {
        let op1 = Operation::Multiply;
        let op2 = op1.clone();
        assert_eq!(op1, op2);
    }

    #[test]
    fn test_calc_error_messages() {
        let err1 = CalcError::DivisionByZero;
        let err2 = CalcError::NegativeSqrt;
        let err3 = CalcError::Overflow;
        
        assert_eq!(err1.as_str(), "Division by zero");
        assert_eq!(err2.as_str(), "Cannot take square root of negative number");
        assert_eq!(err3.as_str(), "Factorial overflow: n must be <= 20");
    }

    #[test]
    fn test_calc_error_equality() {
        let err1 = CalcError::DivisionByZero;
        let err2 = CalcError::DivisionByZero;
        let err3 = CalcError::NegativeSqrt;
        
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_calc_error_clone() {
        let err1 = CalcError::Overflow;
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_calculation_history_clone() {
        let history = CalculationHistory {
            operand1: 10.0,
            operand2: 5.0,
            operation: Operation::Add,
            result: 15.0,
        };
        let history2 = history.clone();
        
        assert_eq!(history.operand1, history2.operand1);
        assert_eq!(history.operand2, history2.operand2);
        assert_eq!(history.operation, history2.operation);
        assert_eq!(history.result, history2.result);
    }

    #[test]
    fn test_divide_after_error() {
        let mut calc = Calculator::new();
        calc.set_value(10.0);
        
        // First division by zero
        let result1 = calc.divide(0.0);
        assert!(result1.is_err());
        
        // Value should still be 10.0
        assert_eq!(calc.get_value(), 10.0);
        
        // Valid division should work
        let result2 = calc.divide(2.0);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 5.0);
    }

    #[test]
    fn test_sqrt_after_error() {
        let mut calc = Calculator::new();
        calc.set_value(-4.0);
        
        // Sqrt of negative should fail
        let result1 = calc.sqrt();
        assert!(result1.is_err());
        
        // Value should still be -4.0
        assert_eq!(calc.get_value(), -4.0);
        
        // Set positive and sqrt should work
        calc.set_value(16.0);
        let result2 = calc.sqrt();
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 4.0);
    }

    #[test]
    fn test_multiple_operations_on_same_calculator() {
        let mut calc = Calculator::new();
        
        calc.set_value(100.0);
        calc.add(50.0);        // 150
        calc.subtract(30.0);   // 120
        calc.divide(4.0).unwrap(); // 30
        calc.multiply(2.0);    // 60
        
        assert_eq!(calc.get_value(), 60.0);
        assert_eq!(calc.history_count(), 4); // 4 operations (add, subtract, divide, multiply)
    }

    #[test]
    fn test_memory_operations_sequence() {
        let mut calc = Calculator::new();
        
        // Store 100
        calc.set_value(100.0);
        calc.memory_store();
        assert_eq!(calc.get_memory(), 100.0);
        
        // Do some calculations
        calc.set_value(50.0);
        calc.add(25.0); // 75
        
        // Recall should restore memory
        let recalled = calc.memory_recall();
        assert_eq!(recalled, 100.0);
        assert_eq!(calc.get_value(), 100.0);
        
        // Add to memory
        calc.set_value(50.0);
        calc.memory_add();
        assert_eq!(calc.get_memory(), 150.0);
        
        // Clear memory
        calc.memory_clear();
        assert_eq!(calc.get_memory(), 0.0);
    }

    #[test]
    fn test_history_with_all_operations() {
        let mut calc = Calculator::new();
        
        calc.set_value(100.0);
        calc.add(10.0);        // 110
        calc.subtract(10.0);   // 100
        calc.multiply(2.0);    // 200
        calc.divide(4.0).unwrap(); // 50
        
        assert_eq!(calc.history_count(), 4);
        
        calc.clear_history();
        assert_eq!(calc.history_count(), 0);
    }

    #[test]
    fn test_power_with_fraction() {
        let mut calc = Calculator::new();
        calc.set_value(16.0);
        let result = calc.power(0.5); // Square root via power
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_power_with_large_exponent() {
        let mut calc = Calculator::new();
        calc.set_value(2.0);
        let result = calc.power(10.0);
        assert_eq!(result, 1024.0);
    }

    #[test]
    fn test_factorial_edge_cases() {
        assert_eq!(factorial(0).unwrap(), 1);
        assert_eq!(factorial(1).unwrap(), 1);
        assert_eq!(factorial(2).unwrap(), 2);
        assert_eq!(factorial(3).unwrap(), 6);
        assert_eq!(factorial(4).unwrap(), 24);
        assert_eq!(factorial(20).unwrap(), 2432902008176640000);
        
        // Test overflow
        assert!(factorial(21).is_err());
        assert!(factorial(25).is_err());
        assert!(factorial(100).is_err());
    }

    #[test]
    fn test_percentage_zero() {
        assert_eq!(percentage(0.0, 50.0), 0.0);
        assert_eq!(percentage(100.0, 0.0), 0.0);
    }

    #[test]
    fn test_percentage_negative() {
        assert_eq!(percentage(-100.0, 10.0), -10.0);
        assert_eq!(percentage(100.0, -10.0), -10.0);
    }

    #[test]
    fn test_compound_interest_edge_cases() {
        // Zero principal
        let result = compound_interest(0.0, 5.0, 10.0, 12.0);
        assert_eq!(result, 0.0);
        
        // Zero rate
        let result = compound_interest(1000.0, 0.0, 10.0, 12.0);
        assert_eq!(result, 1000.0);
        
        // Zero years
        let result = compound_interest(1000.0, 5.0, 0.0, 12.0);
        assert_eq!(result, 1000.0);
    }

    #[test]
    fn test_divide_very_small_number() {
        let mut calc = Calculator::new();
        calc.set_value(1.0);
        let result = calc.divide(1000000.0).unwrap();
        assert!((result - 0.000001).abs() < 0.0000001);
    }

    #[test]
    fn test_multiply_very_large_numbers() {
        let mut calc = Calculator::new();
        calc.set_value(1e150);
        let result = calc.multiply(1e150);
        // Due to floating point precision, check that result is close to 1e300
        assert!(result > 9e299 && result < 1.1e300);
    }

    #[test]
    fn test_operations_with_infinity() {
        let mut calc = Calculator::new();
        calc.set_value(f64::INFINITY);
        
        let result = calc.add(100.0);
        assert!(result.is_infinite());
        
        calc.set_value(1.0);
        let result = calc.divide(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_resets_all_state() {
        let mut calc = Calculator::new();
        
        calc.set_value(42.0);
        calc.add(10.0);
        calc.memory_store();
        
        calc.clear();
        
        assert_eq!(calc.get_value(), 0.0);
        // Memory should persist after clear
        assert_eq!(calc.get_memory(), 52.0);
    }

    #[test]
    fn test_sqrt_of_one() {
        let mut calc = Calculator::new();
        calc.set_value(1.0);
        let result = calc.sqrt().unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_power_of_one() {
        let mut calc = Calculator::new();
        calc.set_value(99.0);
        let result = calc.power(1.0);
        assert_eq!(result, 99.0);
    }

    #[test]
    fn test_subtract_larger_from_smaller() {
        let mut calc = Calculator::new();
        calc.set_value(5.0);
        let result = calc.subtract(10.0);
        assert_eq!(result, -5.0);
        
        // Can still operate on negative result
        calc.add(20.0);
        assert_eq!(calc.get_value(), 15.0);
    }

    #[test]
    fn test_factorial_sequential() {
        let results: Vec<u64> = (0..=10)
            .map(|n| factorial(n).unwrap())
            .collect();
        
        assert_eq!(results[0], 1);    // 0!
        assert_eq!(results[1], 1);    // 1!
        assert_eq!(results[5], 120);  // 5!
        assert_eq!(results[10], 3628800); // 10!
    }

    #[test]
    fn test_memory_with_zero() {
        let mut calc = Calculator::new();
        
        calc.set_value(0.0);
        calc.memory_store();
        assert_eq!(calc.get_memory(), 0.0);
        
        calc.set_value(100.0);
        let recalled = calc.memory_recall();
        assert_eq!(recalled, 0.0);
    }

    #[test]
    fn test_memory_with_negative() {
        let mut calc = Calculator::new();
        
        calc.set_value(-50.0);
        calc.memory_store();
        assert_eq!(calc.get_memory(), -50.0);
        
        calc.set_value(25.0);
        calc.memory_add();
        assert_eq!(calc.get_memory(), -25.0);
    }
}
