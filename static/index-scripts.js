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
        // Update the fractal parameters as they may have changed.
        // Just update the existing fields.
        document.getElementById('init_rows').value = data.params.value1;
        document.getElementById('init_cols').value = data.params.value2;
        document.getElementById('init_mid_pt_re').value = data.params.value3;
        document.getElementById('init_mid_pt_im').value = data.params.value4;
        document.getElementById('init_pt_div').value = data.params.value5;
        document.getElementById('init_max_its').value = data.params.value6;

        if (data.generation === "True") {

            // Filename of fractal image.
            console.log('Fractal image: :', data.image);

            // Display the generated image
            const imageElement = document.getElementById("fractalImage");
            const imageUrl = `./fractals/${data.image}`;
            document.getElementById("fractalImage").src = imageUrl;
            imageElement.style.display = "block";

            // Time to perform fractal generation.
            console.log('Fractal generated in: :', data.time);

            // Update UI text boxes with status.
            document.getElementById('duration-box').value = data.time;
            document.getElementById('error-box').value = "Success";
        } else {
            throw new Error(data.error);
        }
    })
    .catch(error => {
        console.error('Error:', error);
        // Update UI text boxes with status.
        document.getElementById('duration-box').value = data.time;
        document.getElementById('error-box').value = error.message;
        alert("Failed to generate fractal.");
    });});
