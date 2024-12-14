// Javascript UI functions.

// Access the default values from the `window` object
const defaults = window.defaults;

window.onload = () => {
    document.getElementById('init_rows').value = defaults.value1 !== undefined ? defaults.value1 : '';
    document.getElementById('init_cols').value = defaults.value2 !== undefined ? defaults.value2 : '';
    document.getElementById('init_mid_pt_re').value = defaults.value3 !== undefined ? defaults.value3 : '';
    document.getElementById('init_mid_pt_im').value = defaults.value4 !== undefined ? defaults.value4 : '';
    document.getElementById('init_pt_div').value = defaults.value5 !== undefined ? defaults.value5 : '';
    document.getElementById('init_max_its').value = defaults.value6 !== undefined ? defaults.value6 : '';
};

// Listener for fractal initialisation button pressed.
document.getElementById('initializeButton').addEventListener('click', () => {
    const value1 = parseInt(document.getElementById('init_rows').value);
    const value2 = parseInt(document.getElementById('init_cols').value);
    const value3 = parseFloat(document.getElementById('init_mid_pt_re').value);
    const value4 = parseFloat(document.getElementById('init_mid_pt_im').value);
    const value5 = parseFloat(document.getElementById('init_pt_div').value);
    const value6 = parseInt(document.getElementById('init_max_its').value);

    fetch('/generate', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ value1, value2, value3, value4, value5, value6 }),
    })
    .then(response => response.json())
    .then(data => {
        console.log("Generation data enpoint reached.");
        if (data.generation === "True") {
            console.log('Fractall generated in: :', data.time);
        } else {
            throw new Error(data.error);
        }
    })
    .catch(error => {
        console.error('Error:', error);
        alert("Failed to generate fractal.");
    });});
