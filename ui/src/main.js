const { invoke } = window.__TAURI__.core;

const PAGE_SIZE = 25; // correos que se cargan por vez

window.addEventListener("DOMContentLoaded", () => {
  const status = document.querySelector("#status");

  document.querySelector("#connect-btn").addEventListener("click", async () => {
    status.textContent = "Conectando...";
    try {
      const email = await invoke("connect");
      status.textContent = "Conectado como " + email;
    } catch (error) {
      status.textContent = "Error: " + error;
    }
  });

  const result = document.querySelector("#result");
  const rows = document.querySelector("#rows");
  const moreBtn = document.querySelector("#more-btn");
  const trashBtn = document.querySelector("#trash-btn");
  const selectAll = document.querySelector("#select-all");
  let ids = [];
  let shown = 0;

  // Pide los datos de los siguientes correos y los agrega a la tabla
  async function loadMore() {
    moreBtn.disabled = true;
    try {
      const emails = await invoke("previews", { ids: ids.slice(shown, shown + PAGE_SIZE) });

      for (const email of emails) {
        const box = document.createElement("input");
        box.type = "checkbox";
        box.value = email.id;
        box.checked = selectAll.checked;

        const row = rows.insertRow();
        row.insertCell().append(box);
        row.insertCell().textContent = email.date;
        row.insertCell().textContent = email.from;
        row.insertCell().textContent = email.subject;
      }

      shown += emails.length;
    } catch (error) {
      result.textContent = "Error: " + error;
    }
    moreBtn.disabled = false;
    moreBtn.style.display = shown < ids.length ? "" : "none";
  }

  document.querySelector("#search-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    result.textContent = "Buscando...";
    rows.replaceChildren();
    selectAll.checked = false;
    shown = 0;
    try {
      ids = await invoke("search", { query: document.querySelector("#query").value });
      result.textContent = "Se encontraron " + ids.length + " correos.";
      await loadMore();
    } catch (error) {
      result.textContent = "Error: " + error;
    }
  });

  moreBtn.addEventListener("click", loadMore);

  selectAll.addEventListener("change", () => {
    for (const box of rows.querySelectorAll("input")) {
      box.checked = selectAll.checked;
    }
  });

  // Mueve a la papelera solo los correos con la casilla marcada
  trashBtn.addEventListener("click", async () => {
    const boxes = [...rows.querySelectorAll("input:checked")];

    if (boxes.length === 0) {
      result.textContent = "No hay correos seleccionados.";
      return;
    }

    if (!confirm("¿Mover " + boxes.length + " correo(s) a la papelera?")) {
      return;
    }

    trashBtn.disabled = true;
    result.textContent = "Moviendo a la papelera...";
    try {
      const report = await invoke("trash", { ids: boxes.map((box) => box.value) });
      const failed = new Set(report.failed.map((failure) => failure.id));

      // Quita de la tabla los que si se movieron
      for (const box of boxes) {
        if (!failed.has(box.value)) {
          ids = ids.filter((id) => id !== box.value);
          shown -= 1;
          box.closest("tr").remove();
        }
      }

      selectAll.checked = false;
      result.textContent =
        "Movidos a la papelera: " + report.moved + ". Fallidos: " + report.failed.length + ".";
    } catch (error) {
      result.textContent = "Error: " + error;
    }
    trashBtn.disabled = false;
  });
});