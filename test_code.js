// Test file for ast-grep functionality
function testFunction() {
    // This should trigger console.log warning
    console.log("Hello world");

    // This should trigger strict equality warning
    if (value == null) {
        return false;
    }

    // This should trigger XSS warning
    document.getElementById("content").innerHTML = userInput;

    // Some normal code
    const result = calculateValue(input);

    if (result === expectedValue) {
        return true;
    }

    return result;
}

function calculateValue(input) {
    // Another console.log that should be detected
    console.log("Calculating value for:", input);

    // More == usage
    if (input == undefined) {
        return 0;
    }

    return input * 2;
}

// Arrow function
const arrowFunc = (x, y) => {
    console.log("Arrow function called");
    return x == y ? x : y;
};

// Class definition
class TestClass {
    constructor(name) {
        this.name = name;
    }

    display() {
        // More innerHTML usage
        document.body.innerHTML = `<h1>${this.name}</h1>`;
        console.log("Display method called");
    }
}
