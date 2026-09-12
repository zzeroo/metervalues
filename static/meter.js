async function loadMeter() {
    const meterElement = document.getElementById("meter");

    const params = new URLSearchParams(window.location.search);
    const meterId = params.get("id");

    if (!meterId) {
        meterElement.innerHTML = "<p>No meter ID provided.</p>";
        return;
    }

    try {
        const meterResponse = await fetch(`/api/meters/${meterId}`);

        if (!meterResponse.ok) {
            throw new Error("Could not load meter");
        }

        const meter = await meterResponse.json();

        const instancesResponse = await fetch(
            `/api/meters/${meterId}/instances`
        );

        if (!instancesResponse.ok) {
            throw new Error("Could not load meter instances");
        }

        const instances = await instancesResponse.json();

        meterElement.innerHTML = "";

        const title = document.createElement("h1");
        title.textContent = meter.name;

        const unit = document.createElement("p");
        unit.textContent = `Unit: ${meter.unit}`;

        meterElement.appendChild(title);
        meterElement.appendChild(unit);

        // --------------------------------------------------------
        // Meter instances
        // --------------------------------------------------------

        const instancesTitle = document.createElement("h2");
        instancesTitle.textContent = "Meter instances";

        meterElement.appendChild(instancesTitle);

        if (instances.length === 0) {
            const noInstances = document.createElement("p");
            noInstances.textContent = "No meter instances found.";

            meterElement.appendChild(noInstances);
        } else {
            const instancesGrid = document.createElement("div");
            instancesGrid.className = "meter-instance-grid";

            for (const instance of instances) {
                const instanceCard = document.createElement("div");
                instanceCard.className = "meter-instance-card";

                const number = document.createElement("h3");
                number.textContent = instance.meter_number;

                const installed = document.createElement("p");
                installed.textContent =
                    `Installed: ${instance.installed_at}`;

                const status = document.createElement("p");

                if (instance.removed_at) {
                    status.textContent =
                        `Removed: ${instance.removed_at}`;
                } else {
                    status.textContent = "Status: Active";
                }

                instanceCard.appendChild(number);
                instanceCard.appendChild(installed);
                instanceCard.appendChild(status);

                instancesGrid.appendChild(instanceCard);
            }

            meterElement.appendChild(instancesGrid);
        }

        // --------------------------------------------------------
        // Readings
        // --------------------------------------------------------

        const readingsTitle = document.createElement("h2");
        readingsTitle.textContent = "Readings";

        meterElement.appendChild(readingsTitle);

        const readingsTable = document.createElement("table");
        readingsTable.className = "readings-table";

        const tableHeader = document.createElement("thead");
        const headerRow = document.createElement("tr");

        const dateHeader = document.createElement("th");
        dateHeader.textContent = "Date";

        const instanceHeader = document.createElement("th");
        instanceHeader.textContent = "Meter";

        const valueHeader = document.createElement("th");
        valueHeader.textContent = "Value";

        headerRow.appendChild(dateHeader);
        headerRow.appendChild(instanceHeader);
        headerRow.appendChild(valueHeader);

        tableHeader.appendChild(headerRow);
        readingsTable.appendChild(tableHeader);

        const tableBody = document.createElement("tbody");

        for (const instance of instances) {
            const readingsResponse = await fetch(
                `/api/meter-instances/${instance.id}/readings`
            );

            if (!readingsResponse.ok) {
                throw new Error("Could not load readings");
            }

            const readings = await readingsResponse.json();

            for (const reading of readings) {
                const row = document.createElement("tr");

                const date = document.createElement("td");
                date.textContent = reading.reading_date;

                const meterNumber = document.createElement("td");
                meterNumber.textContent = instance.meter_number;

                const value = document.createElement("td");
                value.textContent = `${reading.value} ${meter.unit}`;

                row.appendChild(date);
                row.appendChild(meterNumber);
                row.appendChild(value);

                tableBody.appendChild(row);
            }
        }

        readingsTable.appendChild(tableBody);

        if (tableBody.children.length === 0) {
            const noReadings = document.createElement("p");
            noReadings.textContent = "No readings found.";

            meterElement.appendChild(noReadings);
        } else {
            meterElement.appendChild(readingsTable);
        }
    } catch (error) {
        console.error(error);

        meterElement.innerHTML =
            "<p>Could not load meter details.</p>";
    }
}

loadMeter();
