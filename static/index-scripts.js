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

// Listener for fractal generate button pressed.
document.getElementById('generateButton').addEventListener('click', () => {
    const value1 = parseInt(document.getElementById('init_rows').value);
    const value2 = parseInt(document.getElementById('init_cols').value);
    const value3 = parseFloat(document.getElementById('init_mid_pt_re').value);
    const value4 = parseFloat(document.getElementById('init_mid_pt_im').value);
    const value5 = parseFloat(document.getElementById('init_pt_div').value);
    const value6 = parseInt(document.getElementById('init_max_its').value);


    // Clear the duration field.
    const durationBox = document.getElementById("duration-box");
    durationBox.value = "";

    // Set the status field to "Pending..."
    const statusBox = document.getElementById("error-box");
    statusBox.value = "Pending...";

    // Post to back-end to generate fractal image.
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

    const recentreButton = document.getElementById("recentreButton");
const fractalImage = document.getElementById("fractalImage");

let isRecentreMode = false;

recentreButton.addEventListener("click", () => {
    isRecentreMode = !isRecentreMode;

    // Toggle crosshair cursor
    if (isRecentreMode) {
        fractalImage.classList.add("crosshair-cursor");
        recentreButton.textContent = "Cancel Recentre";
    } else {
        fractalImage.classList.remove("crosshair-cursor");
        recentreButton.textContent = "Recentre";
    }
});

fractalImage.addEventListener("click", (event) => {
    if (!isRecentreMode) return;

    // Get image and click coordinates
    const rect = fractalImage.getBoundingClientRect();
    const clickX = event.clientX - rect.left;
    const clickY = event.clientY - rect.top;

    // Map pixel coordinates to fractal coordinates
    const fractalWidth = fractalImage.naturalWidth;
    const fractalHeight = fractalImage.naturalHeight;
    const currentCentreRe = parseFloat(document.getElementById("init_mid_pt_re").value);
    const currentCentreIm = parseFloat(document.getElementById("init_mid_pt_im").value);
    const pixelDivision = parseFloat(document.getElementById("init_pt_div").value);

    const newCentreRe = currentCentreRe + (clickX - fractalWidth / 2) * pixelDivision;
    const newCentreIm = currentCentreIm - (clickY - fractalHeight / 2) * pixelDivision;

    // Update input fields with new centre
    document.getElementById("init_mid_pt_re").value = newCentreRe.toFixed(6);
    document.getElementById("init_mid_pt_im").value = newCentreIm.toFixed(6);

    // Exit Recentre mode and regenerate fractal
    isRecentreMode = false;
    fractalImage.classList.remove("crosshair-cursor");
    recentreButton.textContent = "Recentre";
    document.getElementById("initializeButton").click(); // Trigger fractal generation
});
