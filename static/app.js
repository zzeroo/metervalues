async function loadMeters() {
    const metersElement = document.getElementById("meters");

    try {
        const response = await fetch("/api/meters");

        if (!response.ok) {
            throw new Error("Could not load meters");
        }

        const meters = await response.json();

        if (meters.length === 0) {
            metersElement.innerHTML = "<p>No meters found.</p>";
            return;
        }

        const meterGrid = document.createElement("div");
        meterGrid.className = "meter-grid";

        for (const meter of meters) {
            const meterCard = document.createElement("a");

            meterCard.className = "meter-card";
            meterCard.href = `/meter.html?id=${meter.id}`;

            const name = document.createElement("h2");
            name.textContent = meter.name;

            const unit = document.createElement("p");
            unit.textContent = `Unit: ${meter.unit}`;

            const linkText = document.createElement("span");
            linkText.textContent = "View meter →";

            meterCard.appendChild(name);
            meterCard.appendChild(unit);
            meterCard.appendChild(linkText);

            meterGrid.appendChild(meterCard);
        }

        metersElement.innerHTML = "";
        metersElement.appendChild(meterGrid);
    } catch (error) {
        console.error(error);

        metersElement.innerHTML =
            "<p>Could not load meters.</p>";
    }
}

loadMeters();
