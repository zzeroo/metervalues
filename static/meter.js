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

    const instancesResponse = await fetch(`/api/meters/${meterId}/instances`);

    if (!instancesResponse.ok) {
      throw new Error("Could not load meter instances");
    }

    const instances = await instancesResponse.json();

    meterElement.innerHTML = "";

    const title = document.createElement("h1");
    title.textContent = meter.name;

    const unit = document.createElement("p");
    unit.textContent = `Unit: ${meter.unit}`;

    const deleteButton = document.createElement("button");
    deleteButton.textContent = "Delete meter";
    deleteButton.className = "delete-meter-button";
    deleteButton.type = "button";

    deleteButton.addEventListener("click", () => {
      deleteMeter(meter.id, meter.name);
    });

    meterElement.appendChild(title);
    meterElement.appendChild(unit);
    meterElement.appendChild(deleteButton);

    // --------------------------------------------------------
    // New meter instance form
    // --------------------------------------------------------

    const newMeterInstanceLink = document.createElement("a");
    newMeterInstanceLink.href = "#";
    newMeterInstanceLink.textContent = "+ New meter instance";

    const newMeterInstanceSection = document.createElement("div");

    if (instances.length > 0) {
        newMeterInstanceSection.style.display = "none";
    }

    const instanceForm = document.createElement("form");
    instanceForm.className = "meter-instance-form";

    newMeterInstanceLink.addEventListener("click", (event) => {
        event.preventDefault();

        const hidden = newMeterInstanceSection.style.display === "none";

        newMeterInstanceSection.style.display = hidden ? "" : "none";
        newMeterInstanceLink.textContent = hidden
            ? "− New meter instance"
            : "+ New meter instance";
    });

    const numberLabel = document.createElement("label");
    numberLabel.textContent = "Meter number";
    numberLabel.htmlFor = "meter-number";

    const numberInput = document.createElement("input");
    numberInput.id = "meter-number";
    numberInput.name = "meter_number";
    numberInput.type = "text";
    numberInput.required = true;

    const initialReadingLabel = document.createElement("label");
    initialReadingLabel.textContent = `Initial reading (${meter.unit})`;
    initialReadingLabel.htmlFor = "initial-reading";

    const initialReadingInput = document.createElement("input");
    initialReadingInput.id = "initial-reading";
    initialReadingInput.name = "initial_reading";
    initialReadingInput.type = "number";
    initialReadingInput.step = "0.001";
    initialReadingInput.min = "0";
    initialReadingInput.required = true;

    const initialDateLabel = document.createElement("label");
    initialDateLabel.textContent = "Initial reading date";
    initialDateLabel.htmlFor = "initial-reading-date";

    const initialDateInput = document.createElement("input");
    initialDateInput.id = "initial-reading-date";
    initialDateInput.name = "initial_reading_date";
    initialDateInput.type = "date";
    initialDateInput.value = new Date().toISOString().split("T")[0];
    initialDateInput.required = true;

    const installedLabel = document.createElement("label");
    installedLabel.textContent = "Installed at";
    installedLabel.htmlFor = "installed-at";

    const installedInput = document.createElement("input");
    installedInput.id = "installed-at";
    installedInput.name = "installed_at";
    installedInput.type = "date";
    installedInput.value = new Date().toISOString().split("T")[0];
    installedInput.required = true;

    const instanceSubmitButton = document.createElement("button");
    instanceSubmitButton.type = "submit";
    instanceSubmitButton.textContent = "Add meter instance";

    const instanceMessage = document.createElement("p");

    instanceForm.appendChild(numberLabel);
    instanceForm.appendChild(numberInput);

    instanceForm.appendChild(initialReadingLabel);
    instanceForm.appendChild(initialReadingInput);

    instanceForm.appendChild(initialDateLabel);
    instanceForm.appendChild(initialDateInput);

    instanceForm.appendChild(installedLabel);
    instanceForm.appendChild(installedInput);

    instanceForm.appendChild(instanceSubmitButton);
    instanceForm.appendChild(instanceMessage);

    instanceForm.addEventListener("submit", async (event) => {
      event.preventDefault();

      instanceMessage.textContent = "";

      const meterInstance = {
        meter_number: numberInput.value,
        initial_reading: Number(initialReadingInput.value),
        initial_reading_date: initialDateInput.value,
        installed_at: installedInput.value,
      };

      try {
        const response = await fetch(`/api/meters/${meterId}/instances`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(meterInstance),
        });

        if (!response.ok) {
          throw new Error("Could not create meter instance");
        }

        await loadMeter();
      } catch (error) {
        console.error(error);

        instanceMessage.textContent =
          "Could not add meter instance.";
      }
    });

    newMeterInstanceSection.appendChild(instanceForm);

    // --------------------------------------------------------
    // Meter instance CSV import / export
    // --------------------------------------------------------

    const csvSection = document.createElement("div");
    csvSection.className = "csv-section";

    const csvTitle = document.createElement("h2");
    csvTitle.textContent = "CSV import";

    const downloadLink = document.createElement("a");
    downloadLink.href =
        "data:text/csv;charset=utf-8,meter_name%2Cmeter_number%2Cinitial_reading%2Cinitial_reading_date%2Cinstalled_at%2Cremoved_at%0AElectricity%2C47110001%2C12345.678%2C2026-01-01%2C2026-01-01%2C%0A";
    downloadLink.download = "meter-instances.csv";
    downloadLink.textContent = "Download CSV template";

    const importForm = document.createElement("form");

    const fileInput = document.createElement("input");
    fileInput.type = "file";
    fileInput.accept = ".csv,text/csv";
    fileInput.required = true;

    const importButton = document.createElement("button");
    importButton.type = "submit";
    importButton.textContent = "Import meter instances";

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
            const response = await fetch("/api/import/meter-instances", {
                method: "POST",
                body: await file.arrayBuffer(),
            });

            if (!response.ok) {
                throw new Error("Could not import meter instances");
            }

            importMessage.textContent =
                "Meter instances imported successfully.";

            await loadMeter();
        } catch (error) {
            console.error(error);

            importMessage.textContent =
                "Could not import meter instances.";
        }
    });

    csvSection.appendChild(csvTitle);
    csvSection.appendChild(downloadLink);
    csvSection.appendChild(importForm);

    newMeterInstanceSection.appendChild(csvSection);

    if (instances.length > 0) {
        meterElement.appendChild(newMeterInstanceLink);
    }

    meterElement.appendChild(newMeterInstanceSection);

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
      return;
    }

    // Active meter first, then older meters newest to oldest.
    instances.sort((a, b) => {
      const aActive = !a.removed_at;
      const bActive = !b.removed_at;

      if (aActive && !bActive) {
        return -1;
      }

      if (!aActive && bActive) {
        return 1;
      }

      return b.installed_at.localeCompare(a.installed_at);
    });

    const instancesGrid = document.createElement("div");
    instancesGrid.className = "meter-instance-grid";

    let activeInstance = null;

    for (const instance of instances) {
      const instanceCard = document.createElement("div");
      instanceCard.className = "meter-instance-card";

      const number = document.createElement("h3");
      number.textContent = instance.meter_number;

      const installed = document.createElement("p");
      installed.textContent = `Installed: ${instance.installed_at}`;

      const status = document.createElement("p");

      if (instance.removed_at) {
        status.textContent = `Removed: ${instance.removed_at}`;
      } else {
        status.textContent = "Status: Active";
        activeInstance = instance;
      }

      instanceCard.appendChild(number);
      instanceCard.appendChild(installed);
      instanceCard.appendChild(status);

      instancesGrid.appendChild(instanceCard);
    }

    meterElement.appendChild(instancesGrid);

    // --------------------------------------------------------
    // New reading form
    // --------------------------------------------------------

    const readingFormTitle = document.createElement("h2");
    readingFormTitle.textContent = "Add reading";

    meterElement.appendChild(readingFormTitle);

    if (!activeInstance) {
      const noActiveInstance = document.createElement("p");
      noActiveInstance.textContent = "No active meter instance available.";

      meterElement.appendChild(noActiveInstance);
    } else {
      const form = document.createElement("form");
      form.className = "reading-form";

      const dateLabel = document.createElement("label");
      dateLabel.textContent = "Date";
      dateLabel.htmlFor = "reading-date";

      const dateInput = document.createElement("input");
      dateInput.id = "reading-date";
      dateInput.name = "reading_date";
      dateInput.type = "date";
      dateInput.value = new Date().toISOString().split("T")[0];
      dateInput.required = true;

      const valueLabel = document.createElement("label");
      valueLabel.textContent = `Value (${meter.unit})`;
      valueLabel.htmlFor = "reading-value";

      const valueInput = document.createElement("input");
      valueInput.id = "reading-value";
      valueInput.name = "value";
      valueInput.type = "number";
      valueInput.step = "0.001";
      valueInput.min = "0";
      valueInput.required = true;

      const submitButton = document.createElement("button");
      submitButton.type = "submit";
      submitButton.textContent = "Add reading";

      const message = document.createElement("p");

      form.appendChild(dateLabel);
      form.appendChild(dateInput);
      form.appendChild(valueLabel);
      form.appendChild(valueInput);
      form.appendChild(submitButton);
      form.appendChild(message);

      form.addEventListener("submit", async (event) => {
        event.preventDefault();

        message.textContent = "";

        const reading = {
          reading_date: dateInput.value,
          value: Number(valueInput.value),
        };

        try {
          const response = await fetch(
            `/api/meter-instances/${activeInstance.id}/readings`,
            {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
              },
              body: JSON.stringify(reading),
            },
          );

          if (!response.ok) {
            throw new Error("Could not create reading");
          }

          dateInput.value = "";
          valueInput.value = "";

          await loadMeter();
        } catch (error) {
          console.error(error);
          message.textContent = "Could not add reading.";
        }
      });

      meterElement.appendChild(form);
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
        `/api/meter-instances/${instance.id}/readings`,
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

    // Newest reading first.
    const rows = Array.from(tableBody.querySelectorAll("tr"));

    rows.sort((a, b) => {
      const dateA = a.cells[0].textContent;
      const dateB = b.cells[0].textContent;

      return dateB.localeCompare(dateA);
    });

    tableBody.innerHTML = "";

    for (const row of rows) {
      tableBody.appendChild(row);
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

    meterElement.innerHTML = "<p>Could not load meter details.</p>";
  }
}

async function deleteMeter(meterId, meterName) {
  const confirmed = window.confirm(
    `Delete meter "${meterName}" and all its readings?\n\nThis cannot be undone.`,
  );

  if (!confirmed) {
    return;
  }

  try {
    const response = await fetch(`/api/meters/${meterId}`, {
      method: "DELETE",
    });

    if (!response.ok) {
      throw new Error("Could not delete meter");
    }

    window.location.href = "/";
  } catch (error) {
    console.error(error);

    alert("Could not delete meter.");
  }
}

loadMeter();
