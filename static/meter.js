async function loadMeter() {
    const meterElement = document.getElementById("meter");

    const params = new URLSearchParams(window.location.search);
    const meterId = params.get("id");

    if (!meterId) {
        meterElement.innerHTML = "<p>No meter ID provided.</p>";
        return;
    }

    try {
        const response = await fetch(`/api/meters/${meterId}`);

        if (!response.ok) {
            throw new Error("Could not load meter");
        }

        const meter = await response.json();

        const title = document.createElement("h1");
        title.textContent = meter.name;

        const unit = document.createElement("p");
        unit.textContent = `Unit: ${meter.unit}`;

        meterElement.innerHTML = "";
        meterElement.appendChild(title);
        meterElement.appendChild(unit);
    } catch (error) {
        console.error(error);

        meterElement.innerHTML =
            "<p>Could not load meter.</p>";
    }
}

loadMeter();
