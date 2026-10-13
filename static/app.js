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

        const newMeterSection = document.createElement("div");

        if (meters.length > 0) {
            newMeterSection.style.display = "none";
        }

        const form = document.createElement("form");
        form.className = "meter-form";

        newMeterLink.addEventListener("click", (event) => {
            event.preventDefault();

            const hidden = newMeterSection.style.display === "none";

            newMeterSection.style.display = hidden ? "" : "none";
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

        newMeterSection.appendChild(form);

        if (meters.length > 0) {
            metersElement.appendChild(newMeterLink);
        }

        metersElement.appendChild(newMeterSection);

      // --------------------------------------------------------
      // Meter CSV import / export
      // --------------------------------------------------------

      const csvSection = document.createElement("div");
      csvSection.className = "csv-section";

      const csvTitle = document.createElement("h2");
      csvTitle.textContent = "CSV import";

      const downloadLink = document.createElement("a");
      downloadLink.href = "data:text/csv;charset=utf-8,name%2Cunit%0AElectricity%2CkWh%0AWater%2Cm%C2%B3%0AGas%2Cm%C2%B3%0A";
      downloadLink.download = "meters.csv";
      downloadLink.textContent = "Download CSV template";

      const importForm = document.createElement("form");

      const fileInput = document.createElement("input");
      fileInput.type = "file";
      fileInput.accept = ".csv,text/csv";
      fileInput.required = true;

      const importButton = document.createElement("button");
      importButton.type = "submit";
      importButton.textContent = "Import meters";

      const importMessage = document.createElement("p");

      importForm.appendChild(fileInput);
      importForm.appendChild(importButton);
      importForm.appendChild(importMessage);

      importForm.addEventListener("submit", async (event) => {
          event.preventDefault();

          importMessage.textContent = "";

          const file = fileInput.files[0];

          if (!file) {
              return;
          }

          try {
              const response = await fetch("/api/import/meters", {
                  method: "POST",
                  body: await file.arrayBuffer(),
              });

              if (!response.ok) {
                  throw new Error("Could not import meters");
              }

              importMessage.textContent = "Meters imported successfully.";

              await loadMeters();
          } catch (error) {
              console.error(error);

              importMessage.textContent = "Could not import meters.";
          }
      });

      csvSection.appendChild(csvTitle);
      csvSection.appendChild(downloadLink);
      csvSection.appendChild(importForm);

      newMeterSection.appendChild(csvSection);

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

      // --------------------------------------------------------
      // Meter reading history
      // --------------------------------------------------------

      const readingsSection = document.createElement("div");
      readingsSection.className = "meter-readings-section";

      for (const meter of meters) {
          const meterTitle = document.createElement("h2");
          meterTitle.textContent = meter.name;

          readingsSection.appendChild(meterTitle);

          const instancesResponse = await fetch(
              `/api/meters/${meter.id}/instances`,
          );

          if (!instancesResponse.ok) {
              throw new Error("Could not load meter instances");
          }

          const instances = await instancesResponse.json();

          const readings = [];
          const readingsByInstance = [];

          for (const instance of instances) {
              const readingsResponse = await fetch(
                  `/api/meter-instances/${instance.id}/readings`,
              );

              if (!readingsResponse.ok) {
                  throw new Error("Could not load readings");
              }

              const instanceReadings = await readingsResponse.json();

              readingsByInstance.push({
                  meter_number: instance.meter_number,
                  initial_reading: instance.initial_reading,
                  initial_reading_date: instance.initial_reading_date,
                  readings: instanceReadings,
              });

              for (const reading of instanceReadings) {
                  readings.push({
                      reading_date: reading.reading_date,
                      meter_number: instance.meter_number,
                      value: reading.value,
                  });
              }
          }

          // Newest reading first.
          readings.sort((a, b) => {
              return b.reading_date.localeCompare(a.reading_date);
          });

          if (readings.length === 0) {
              const noReadings = document.createElement("p");
              noReadings.textContent = "No readings found.";

              readingsSection.appendChild(noReadings);

              continue;
          }

//           const table = document.createElement("table");
//           table.className = "readings-table";
//
//           const thead = document.createElement("thead");
//           const headerRow = document.createElement("tr");
//
//           const dateHeader = document.createElement("th");
//           dateHeader.textContent = "Date";
//
//           const meterHeader = document.createElement("th");
//           meterHeader.textContent = "Meter";
//
//           const valueHeader = document.createElement("th");
//           valueHeader.textContent = "Reading";
//
//           headerRow.appendChild(dateHeader);
//           headerRow.appendChild(meterHeader);
//           headerRow.appendChild(valueHeader);
//
//           thead.appendChild(headerRow);
//           table.appendChild(thead);
//
//           const tbody = document.createElement("tbody");
//
//           for (const reading of readings.slice(0, 5)) {
//               const row = document.createElement("tr");
//
//               const date = document.createElement("td");
//               date.textContent = reading.reading_date;
//
//               const meterNumber = document.createElement("td");
//               meterNumber.textContent = reading.meter_number;
//
//               const value = document.createElement("td");
//               value.textContent = `${reading.value} ${meter.unit}`;
//
//               row.appendChild(date);
//               row.appendChild(meterNumber);
//               row.appendChild(value);
//
//               tbody.appendChild(row);
//           }
//
//           table.appendChild(tbody);
//         readingsSection.appendChild(table);

        const usageChart = createUsageChart(
            meter,
            readingsByInstance,
        );

        readingsSection.appendChild(usageChart);
      }

      metersElement.appendChild(readingsSection);

    } catch (error) {
        console.error(error);

        metersElement.innerHTML =
            "<p>Could not load meters.</p>";
    }
}

function createUsageChart(meter, readingsByInstance) {
    const section = document.createElement("div");
    section.className = "usage-section";

    const title = document.createElement("h2");
    title.textContent = `${meter.name} usage`;

    section.appendChild(title);

    const usage = [];

    for (const instance of readingsByInstance) {
        const readings = [...instance.readings].sort((a, b) =>
            a.reading_date.localeCompare(b.reading_date),
        );

        if (readings.length === 0) {
            continue;
        }

        let previousValue = Number(instance.initial_reading);
        let previousDate = instance.initial_reading_date;

        for (const reading of readings) {
            const value = Number(reading.value);

            const consumption = value - previousValue;

            if (consumption >= 0) {
                usage.push({
                    date: reading.reading_date,
                    value: consumption,
                });
            }

            previousValue = value;
            previousDate = reading.reading_date;
        }
    }

    if (usage.length === 0) {
        const message = document.createElement("p");
        message.textContent = "Not enough readings for a usage chart.";
        section.appendChild(message);

        return section;
    }

    usage.sort((a, b) =>
        a.date.localeCompare(b.date),
    );

    // --------------------------------------------------------
    // Summary
    // --------------------------------------------------------

    const latest = usage[usage.length - 1];

    const latestCard = document.createElement("div");
    latestCard.className = "usage-card";

    const latestLabel = document.createElement("span");
    latestLabel.textContent = "Last usage";

    const latestValue = document.createElement("strong");
    latestValue.textContent =
        `${latest.value.toFixed(3)} ${meter.unit}`;

    latestCard.appendChild(latestLabel);
    latestCard.appendChild(latestValue);

    // Last 12 months
    const now = new Date();
    const yearAgo = new Date(now);
    yearAgo.setFullYear(yearAgo.getFullYear() - 1);

    const lastYearUsage = usage
        .filter((entry) => new Date(entry.date) >= yearAgo)
        .reduce((sum, entry) => sum + entry.value, 0);

    const yearCard = document.createElement("div");
    yearCard.className = "usage-card";

    const yearLabel = document.createElement("span");
    yearLabel.textContent = "Last 12 months";

    const yearValue = document.createElement("strong");
    yearValue.textContent =
        `${lastYearUsage.toFixed(3)} ${meter.unit}`;

    yearCard.appendChild(yearLabel);
    yearCard.appendChild(yearValue);

    const summary = document.createElement("div");
    summary.className = "usage-summary";

    summary.appendChild(latestCard);
    summary.appendChild(yearCard);

    section.appendChild(summary);

    // --------------------------------------------------------
    // Monthly usage
    // --------------------------------------------------------

    const monthly = new Map();

    for (const entry of usage) {
        const month = entry.date.substring(0, 7);

        monthly.set(
            month,
            (monthly.get(month) || 0) + entry.value,
        );
    }

    const monthlyEntries = Array.from(monthly.entries())
        .sort(([a], [b]) => a.localeCompare(b));

    if (monthlyEntries.length === 0) {
        return section;
    }

    // --------------------------------------------------------
    // SVG chart
    // --------------------------------------------------------

    const chartWidth = 800;
    const chartHeight = 300;

    const padding = {
        top: 20,
        right: 20,
        bottom: 40,
        left: 60,
    };

    const width =
        chartWidth - padding.left - padding.right;

    const height =
        chartHeight - padding.top - padding.bottom;

    const maxValue = Math.max(
        ...monthlyEntries.map(([, value]) => value),
        1,
    );

    const svg = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "svg",
    );

    svg.setAttribute("viewBox", `0 0 ${chartWidth} ${chartHeight}`);
    svg.classList.add("usage-chart");

    const points = monthlyEntries.map(([month, value], index) => {
        const x =
            padding.left +
            (index / Math.max(monthlyEntries.length - 1, 1)) * width;

        const y =
            padding.top +
            height -
            (value / maxValue) * height;

        return {
            month,
            value,
            x,
            y,
        };
    });

    // Grid
    const axis = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "line",
    );

    axis.setAttribute("x1", padding.left);
    axis.setAttribute("y1", padding.top + height);
    axis.setAttribute("x2", padding.left + width);
    axis.setAttribute("y2", padding.top + height);

    axis.classList.add("chart-axis");

    svg.appendChild(axis);

    // Line
    const path = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "path",
    );

    const pathData = points
        .map((point, index) =>
            `${index === 0 ? "M" : "L"} ${point.x} ${point.y}`,
        )
        .join(" ");

    path.setAttribute("d", pathData);
    path.classList.add("usage-line");

    svg.appendChild(path);

    // Points
    for (const point of points) {
        const circle = document.createElementNS(
            "http://www.w3.org/2000/svg",
            "circle",
        );

        circle.setAttribute("cx", point.x);
        circle.setAttribute("cy", point.y);
        circle.setAttribute("r", "4");

        circle.classList.add("usage-point");

        const title = document.createElementNS(
            "http://www.w3.org/2000/svg",
            "title",
        );

        title.textContent =
            `${point.month}: ${point.value.toFixed(3)} ${meter.unit}`;

        circle.appendChild(title);

        svg.appendChild(circle);
    }

    section.appendChild(svg);

    return section;
}

loadMeters();
