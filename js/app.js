// Import WASM modules
import init, { 
    Calculator, 
    percentage, 
    compound_interest, 
    factorial_js 
} from '../pkg/wasm_calc.js';

// Application state
let calc;
let currentInput = '0';
let operation = null;
let waitingForOperand = false;
let unifiedHistory = [];

// Initialize WASM
async function initWasm() {
    await init();
    calc = new Calculator();
    updateDisplay();
    updateMemoryDisplay();
    checkLogo();
    console.log('WASM Calculator initialized');
}

// Check if logo exists and show it
function checkLogo() {
    const logo = document.getElementById('logo');
    const img = new Image();
    img.onload = function() {
        logo.classList.add('visible');
    };
    img.onerror = function() {
        logo.style.display = 'none';
    };
    img.src = logo.src;
}

// Update display
function updateDisplay() {
    document.getElementById('display').textContent = currentInput;
    updateHistoryDisplay();
}

// Update memory display
function updateMemoryDisplay() {
    const memory = calc.get_memory();
    document.getElementById('memory-display').textContent = 
        `Memory: ${memory !== 0 ? memory : '0'}`;
}

// Update history display
function updateHistoryDisplay() {
    const historyList = document.getElementById('history-list');
    
    document.getElementById('history-count').textContent = unifiedHistory.length;
    
    if (unifiedHistory.length === 0) {
        historyList.innerHTML = '<div style="color: #999; font-style: italic;">No history yet</div>';
        return;
    }
    
    // Display last 5 items in reverse chronological order (most recent first)
    historyList.innerHTML = unifiedHistory
        .slice(-5)
        .reverse()
        .map(item => `<div class="history-item">${item}</div>`)
        .join('');
}

// Append number to input
window.appendNumber = function(num) {
    if (waitingForOperand) {
        currentInput = num;
        waitingForOperand = false;
    } else {
        currentInput = currentInput === '0' ? num : currentInput + num;
    }
    updateDisplay();
};

// Set operation
window.setOperation = function(op) {
    const inputValue = parseFloat(currentInput);
    
    if (op === 'sqrt') {
        calc.set_value(inputValue);
        try {
            const result = calc.sqrt();
            
            // Add to unified history
            unifiedHistory.push(`√${inputValue} = ${result}`);
            
            currentInput = result.toString();
        } catch (error) {
            currentInput = 'Error: ' + error;
        }
        updateDisplay();
        return;
    }
    
    if (op === 'power') {
        calc.set_value(inputValue);
        const result = calc.power(2);
        
        // Add to unified history
        unifiedHistory.push(`${inputValue}² = ${result}`);
        
        currentInput = result.toString();
        updateDisplay();
        return;
    }
    
    if (operation !== null) {
        calculate();
    } else {
        calc.set_value(inputValue);
    }
    
    operation = op;
    waitingForOperand = true;
};

// Calculate result
window.calculate = function() {
    if (operation === null) return;
    
    const inputValue = parseFloat(currentInput);
    const previousValue = calc.get_value();
    let result;
    
    try {
        let opSymbol;
        switch (operation) {
            case 'add':
                result = calc.add(inputValue);
                opSymbol = '+';
                break;
            case 'subtract':
                result = calc.subtract(inputValue);
                opSymbol = '-';
                break;
            case 'multiply':
                result = calc.multiply(inputValue);
                opSymbol = '×';
                break;
            case 'divide':
                result = calc.divide(inputValue);
                opSymbol = '÷';
                break;
        }
        
        // Add to unified history
        unifiedHistory.push(`${previousValue} ${opSymbol} ${inputValue} = ${result}`);
        
        currentInput = result.toString();
        operation = null;
        waitingForOperand = true;
    } catch (error) {
        currentInput = 'Error: ' + error;
    }
    
    updateDisplay();
};

// Clear calculator
window.clearCalc = function() {
    currentInput = '0';
    operation = null;
    waitingForOperand = false;
    calc.clear();
    updateDisplay();
};

// Memory store
window.memoryStore = function() {
    calc.set_value(parseFloat(currentInput));
    calc.memory_store();
    updateMemoryDisplay();
};

// Memory recall
window.memoryRecall = function() {
    const value = calc.memory_recall();
    currentInput = value.toString();
    updateDisplay();
};

// Memory clear
window.memoryClear = function() {
    calc.memory_clear();
    updateMemoryDisplay();
};

// Memory add
window.memoryAdd = function() {
    calc.set_value(parseFloat(currentInput));
    calc.memory_add();
    updateMemoryDisplay();
};

// Clear history
window.clearHistory = function() {
    calc.clear_history();
    unifiedHistory = [];
    updateHistoryDisplay();
};

// Calculate factorial
window.calculateFactorial = function() {
    const n = parseInt(currentInput);
    if (isNaN(n) || n < 0) {
        currentInput = 'Error: Invalid input';
    } else {
        try {
            const result = factorial_js(n);
            
            // Add to unified history
            unifiedHistory.push(`${n}! = ${result}`);
            
            currentInput = result.toString();
        } catch (error) {
            currentInput = 'Error: ' + error;
        }
    }
    updateDisplay();
};

// Initialize on load
initWasm();
