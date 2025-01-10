// Javascript UI functions.

// Access the default values from the `window` object.
const defaults = window.defaults;

window.onload = () => {
    // Initialise parameter values.
    document.getElementById('init_rows').value = defaults.value1 !== undefined ? defaults.value1 : '';
    document.getElementById('init_cols').value = defaults.value2 !== undefined ? defaults.value2 : '';
    document.getElementById('init_mid_pt_re').value = defaults.value3 !== undefined ? defaults.value3 : '';
    document.getElementById('init_mid_pt_im').value = defaults.value4 !== undefined ? defaults.value4 : '';
    document.getElementById('init_pt_div').value = defaults.value5 !== undefined ? defaults.value5 : '';
    document.getElementById('init_max_its').value = defaults.value6 !== undefined ? defaults.value6 : '';
};

// Get referemces to buttons (as needed).
const recentreButton = document.getElementById("recentreButton");
const histogramButton = document.getElementById("histogramButton");

// Intially disable the re-centre and histogram buttons.
window.addEventListener("load", () => {
    recentreButton.disabled = true;
    histogramButton.disabled = true;
});

// Get reference to fractal image (as needed).
const fractalImage = document.getElementById("fractalImage");

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

    // Set the status field to "Pending..." while we wait for back-end to process.
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
        console.log("Generate data endpoint reached.");
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

            // Display the generated image.
            const imageElement = document.getElementById("fractalImage");
            const imageUrl = `./fractals/${data.image}`;
            document.getElementById("fractalImage").src = imageUrl;
            imageElement.style.display = "block";

            // Time to perform fractal generation.
            console.log('Fractal generated in: :', data.time);

            // Update UI text boxes with status.
            document.getElementById('duration-box').value = data.time;
            document.getElementById('error-box').value = "Fractal generation successful.";

            // Enable the re-centre and histogram buttons.
            recentreButton.disabled = false;
            histogramButton.disabled = false;

        } else {
            throw new Error(data.error);
        }
    })
    .catch(error => {
        console.error('Error:', error);
        // Update UI text boxes with status.
        document.getElementById('error-box').value = error.message;
        alert("Failed to generate fractal.");
    });});

let isRecentreMode = false;

// Listener if user selects to recentre the image.
recentreButton.addEventListener("click", () => {
    isRecentreMode = !isRecentreMode;

    // Toggle crosshair cursor.
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

    // Get image and click coordinates.
    const rect = fractalImage.getBoundingClientRect();
    const centre_col = event.clientX - rect.left;
    const centre_row = event.clientY - rect.top;
    console.log(`Initial centre point: row:${centre_row}, col:${centre_col}`);

    const value1 = parseInt(document.getElementById('init_rows').value);
    const value2 = parseInt(document.getElementById('init_cols').value);
    const value3 = parseFloat(document.getElementById('init_mid_pt_re').value);
    const value4 = parseFloat(document.getElementById('init_mid_pt_im').value);
    const value5 = parseFloat(document.getElementById('init_pt_div').value);
    const value6 = parseInt(document.getElementById('init_max_its').value);

    // Map pixel coordinates to fractal coordinates.
    const fractalWidth = fractalImage.naturalWidth;
    const fractalHeight = fractalImage.naturalHeight;
    console.log(`Fractal dimensions: width"${fractalWidth}, height${fractalHeight}`);

    const currentCentreRe = parseFloat(document.getElementById("init_mid_pt_re").value);
    const currentCentreIm = parseFloat(document.getElementById("init_mid_pt_im").value);
    const pixelDivision = parseFloat(document.getElementById("init_pt_div").value);

    const new_centre_re = currentCentreRe + ((centre_col - (fractalWidth / 2))) * pixelDivision;
    const new_centre_im = currentCentreIm + (((fractalHeight / 2) - centre_row)) * pixelDivision;
    console.log(`New centre point: x:${new_centre_re}, y:${new_centre_im}`);

    // Update input fields with new centre.
    document.getElementById("init_mid_pt_re").value = new_centre_re.toFixed(6);
    document.getElementById("init_mid_pt_im").value = new_centre_im.toFixed(6);

    // Log the selected (new) row and column.
    console.log(`Selected centre point: row:${centre_row}, col:${centre_col}`);

    // Clear the duration field.
    const durationBox = document.getElementById("duration-box");
    durationBox.value = "";

    // Set the status field to "Pending..." while we wait for back-end to process.
    const statusBox = document.getElementById("error-box");
    statusBox.value = "Pending...";
    
    // Exit Recentre mode and (optionally) regenerate fractal.
    isRecentreMode = false;
    fractalImage.classList.remove("crosshair-cursor");
    recentreButton.textContent = "Recentre";

    // Post to back-end to recentre fractal image.
    fetch('/recentre', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        // Pass centre point as row and column, and also as real and imaginary coordinates.
        body: JSON.stringify({ centre_row, centre_col, new_centre_re, new_centre_im }),
    })
    .then(response => response.json())
    .then(data => {
        console.log("Recentre image endpoint reached.");

        if (data.recentred === "True") {

            // Filename of fractal image.
            console.log('Fractal image: :', data.image);

            // Display the generated (recentred) image.
            const imageElement = document.getElementById("fractalImage");
            const imageUrl = `./fractals/${data.image}`;
            document.getElementById("fractalImage").src = imageUrl;
            imageElement.style.display = "block";

            // Time to perform fractal recentre and generation.
            console.log('Fractal recentre and generated in: :', data.time);

            // Update UI text boxes with status.
            document.getElementById('duration-box').value = data.time;
            document.getElementById('error-box').value = "Recentring successful.";
        } else {
            throw new Error(data.error);
        }
    })
    .catch(error => {
        console.error('Error:', error);
        // Update UI text boxes with status.
        document.getElementById('error-box').value = error.message;
        alert("Failed to recentre and generate fractal.");
    })
});

const times2Button = document.getElementById("times2Button");
const times3Button = document.getElementById("times3Button");
const times5Button = document.getElementById("times5Button");
const times10Button = document.getElementById("times10Button");

// Listener for fractal zoom x 2 button pressed.
document.getElementById('times2Button').addEventListener('click', () => {

    // Get current value of pixel division and divide by factor.
    // Dividing pixel division will give corresponding zoom be that factor.
    const value5 = parseFloat(document.getElementById('init_pt_div').value);
    const newZoomValue = value5 / 2.0;
    // Update the pixel division.
    document.getElementById('init_pt_div').value = newZoomValue;
});

// Listener for fractal zoom x 3 button pressed.
document.getElementById('times3Button').addEventListener('click', () => {

    // Get current value of pixel division and divide by factor.
    // Dividing pixel division will give corresponding zoom be that factor.
    const value5 = parseFloat(document.getElementById('init_pt_div').value);
    const newZoomValue = value5 / 3.0;
    // Update the pixel division.
    document.getElementById('init_pt_div').value = newZoomValue;
});

// Listener for fractal zoom x 5 button pressed.
document.getElementById('times5Button').addEventListener('click', () => {

    // Get current value of pixel division and divide by factor.
    // Dividing pixel division will give corresponding zoom be that factor.
    const value5 = parseFloat(document.getElementById('init_pt_div').value);
    const newZoomValue = value5 / 5.0;
    // Update the pixel division.
    document.getElementById('init_pt_div').value = newZoomValue;
});

// Listener for fractal zoom x 10 button pressed.
document.getElementById('times10Button').addEventListener('click', () => {

    // Get current value of pixel division and divide by factor.
    // Dividing pixel division will give corresponding zoom be that factor.
    const value5 = parseFloat(document.getElementById('init_pt_div').value);
    const newZoomValue = value5 / 10.0;
    // Update the pixel division.
    document.getElementById('init_pt_div').value = newZoomValue;
});

// Listener for Divergence Histogram button pressed.
document.getElementById('histogramButton').addEventListener('click', () => {

    // Clear the duration field.
    const durationBox = document.getElementById("duration-box");
    durationBox.value = "";

    // Set the status field to "Pending..." while we wait for back-end to process.
    const statusBox = document.getElementById("error-box");
    statusBox.value = "Pending...";

    // Post to back-end to generate fractal divergence histogram data.
    fetch('/histogram', {
        method: 'GET',
    })
    .then(response => response.json())
    .then(data => {
        console.log("Divergence histogram chart endpoint reached.");

        // Parse payload string to json format,
        const jsonPayload = JSON.parse(data.chart);

        if (data.histogram === "True") {

            // Use local storage to hold payload.
            localStorage.setItem('histogramData', JSON.stringify({
                bins: jsonPayload.bins,
                counts: jsonPayload.counts
            }));
            window.open('/static/histogram.html', '_blank');
        } else {
            throw new Error(data.error);
        }
    });
});

// Listener for select active colour palette button pressed.
document.getElementById('paletteButton').addEventListener('click', () => {
    const fileInput = document.getElementById('paletteFileInput');
    fileInput.click();

    fileInput.addEventListener('change', () => {
        if (fileInput.files.length > 0) {
            const selectedFile = fileInput.files[0];
            const formData = new FormData();
            formData.append('palette_file', selectedFile);

            // Post to back-end to upload the palette file
            fetch('/palette', {
                method: 'POST',
                body: formData,
            })
            .then(response => response.json())
            .then(data => {
                if (data.palette === "True") {
                    document.getElementById('palette-box').value = data.palette_file;
                } else {
                    throw new Error(data.error);
                }
            })
            .catch(error => {
                console.error('Error:', error);
                document.getElementById('error-box').value = error.message;
                alert("Failed to upload and set active colour palette.");
            });
        }
    }, { once: true });
});
