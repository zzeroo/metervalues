async function loadMeters() {
    const metersElement = document.getElementById("meters");

    try {
        const response = await fetch("/api/meters");

        if (!response.ok) {
            throw new Error("Could not load meters");
        }

        const meters = await response.json();

        metersElement.innerHTML = "";

        // --------------------------------------------------------
        // New meter form
        // --------------------------------------------------------

        const newMeterLink = document.createElement("a");
        newMeterLink.href = "#";
        newMeterLink.textContent = "+ New meter";

        const form = document.createElement("form");
        form.className = "meter-form";

        if (meters.length > 0) {
            form.style.display = "none";
        }

        newMeterLink.addEventListener("click", (event) => {
            event.preventDefault();

            const hidden = form.style.display === "none";

            form.style.display = hidden ? "" : "none";
            newMeterLink.textContent = hidden
                ? "− New meter"
                : "+ New meter";
        });
        const nameLabel = document.createElement("label");
        nameLabel.textContent = "Name";
        nameLabel.htmlFor = "meter-name";

        const nameInput = document.createElement("input");
        nameInput.id = "meter-name";
        nameInput.name = "name";
        nameInput.type = "text";
        nameInput.required = true;

        const unitLabel = document.createElement("label");
        unitLabel.textContent = "Unit";
        unitLabel.htmlFor = "meter-unit";

        const unitSelect = document.createElement("select");
        unitSelect.id = "meter-unit";
        unitSelect.name = "unit";
        unitSelect.required = true;

        const units = [
            "m³",
            "kWh",
            "Custom..."
        ];

        for (const unit of units) {
            const option = document.createElement("option");
            option.value = unit;
            option.textContent = unit;

            unitSelect.appendChild(option);
        }

        const customUnitInput = document.createElement("input");
        customUnitInput.id = "custom-unit";
        customUnitInput.name = "custom_unit";
        customUnitInput.type = "text";
        customUnitInput.placeholder = "Custom unit";
        customUnitInput.hidden = true;

        unitSelect.addEventListener("change", () => {
            const custom = unitSelect.value === "Custom...";

            customUnitInput.hidden = !custom;
            customUnitInput.required = custom;
        });

        const submitButton = document.createElement("button");
        submitButton.type = "submit";
        submitButton.textContent = "Create meter";

        const message = document.createElement("p");

        form.appendChild(nameLabel);
        form.appendChild(nameInput);
        form.appendChild(unitLabel);
        form.appendChild(unitSelect);
        form.appendChild(customUnitInput);
        form.appendChild(submitButton);
        form.appendChild(message);

        form.addEventListener("submit", async (event) => {
            event.preventDefault();

            message.textContent = "";

            const unit =
                unitSelect.value === "Custom..."
                    ? customUnitInput.value.trim()
                    : unitSelect.value;

            const meter = {
                name: nameInput.value.trim(),
                unit: unit,
            };

            try {
                const response = await fetch("/api/meters", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify(meter),
                });

                if (!response.ok) {
                    throw new Error("Could not create meter");
                }

                await loadMeters();
            } catch (error) {
                console.error(error);

                message.textContent = "Could not create meter.";
            }
        });

        if (meters.length > 0) {
            metersElement.appendChild(newMeterLink);
        }

      metersElement.appendChild(form);

        // --------------------------------------------------------
        // Meter list
        // --------------------------------------------------------

        if (meters.length === 0) {
            const noMeters = document.createElement("p");
            noMeters.textContent = "No meters found.";

            metersElement.appendChild(noMeters);
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

        metersElement.appendChild(meterGrid);
    } catch (error) {
        console.error(error);

        metersElement.innerHTML =
            "<p>Could not load meters.</p>";
    }
}

loadMeters();
