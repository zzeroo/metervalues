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

        const instancesTitle = document.createElement("h2");
        instancesTitle.textContent = "Meter instances";

        meterElement.appendChild(title);
        meterElement.appendChild(unit);
        meterElement.appendChild(instancesTitle);

        if (instances.length === 0) {
            const noInstances = document.createElement("p");
            noInstances.textContent = "No meter instances found.";

            meterElement.appendChild(noInstances);
            return;
        }

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
    } catch (error) {
        console.error(error);

        meterElement.innerHTML =
            "<p>Could not load meter details.</p>";
    }
}

loadMeter();
