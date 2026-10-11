const { invoke } = window.__TAURI__.core;

const PAGE_SIZE = 25; // correos que se cargan por vez

// "Fri, 10 Oct 2025 17:37:04 +0000" -> "10 oct 2025"
function formatDate(text) {
  const date = new Date(text);
  if (isNaN(date)) {
    return text;
  }
  return date.toLocaleDateString("es", { day: "numeric", month: "short", year: "numeric" });
}

// "LinkedIn <jobs@linkedin.com>" -> "LinkedIn"
function senderName(from) {
  const name = from.replace(/<.*>/, "").replaceAll('"', "").trim();
  return name || from;
}

window.addEventListener("DOMContentLoaded", () => {
  const status = document.querySelector("#status");
  const connectBtn = document.querySelector("#connect-btn");
  const queryInput = document.querySelector("#query");
  const result = document.querySelector("#result");
  const selected = document.querySelector("#selected");
  const rows = document.querySelector("#rows");
  const moreBtn = document.querySelector("#more-btn");
  const trashBtn = document.querySelector("#trash-btn");
  const selectAll = document.querySelector("#select-all");
  let ids = [];
  let shown = 0;

  function showMessage(text, isError = false) {
    result.textContent = text;
    result.classList.toggle("error", isError);
  }

  async function connect() {
    status.textContent = "Conectando…";
    status.className = "badge";
    try {
      status.textContent = await invoke("connect");
      status.className = "badge ok";
      connectBtn.style.display = "none";
    } catch (error) {
      status.textContent = "Sin conectar";
      connectBtn.style.display = "";
      showMessage("No se pudo conectar: " + error, true);
    }
  }

  // Resalta las filas marcadas y actualiza el contador y el boton de papelera
  function updateSelection() {
    let count = 0;
    for (const box of rows.querySelectorAll("input")) {
      box.closest("tr").classList.toggle("selected", box.checked);
      if (box.checked) {
        count += 1;
      }
    }
    selected.textContent = count > 0 ? count + " seleccionados" : "";
    trashBtn.disabled = count === 0;
  }

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

        const fromCell = row.insertCell();
        fromCell.textContent = senderName(email.from);
        fromCell.title = email.from;

        const subjectCell = row.insertCell();
        subjectCell.textContent = email.subject;
        subjectCell.title = email.subject;

        row.insertCell().textContent = formatDate(email.date);
      }

      shown += emails.length;
    } catch (error) {
      showMessage("Error: " + error, true);
    }

    const remaining = ids.length - shown;
    moreBtn.disabled = false;
    moreBtn.textContent = "Cargar " + Math.min(PAGE_SIZE, remaining) + " más";
    moreBtn.style.display = remaining > 0 ? "" : "none";
    updateSelection();
  }

  connectBtn.addEventListener("click", connect);

  // Si ya se autorizo antes, conecta sin esperar al boton
  invoke("is_authorized").then((authorized) => {
    if (authorized) {
      connect();
    }
  });

  document.querySelector("#search-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    showMessage("Buscando…");
    rows.replaceChildren();
    selectAll.checked = false;
    ids = [];
    shown = 0;
    updateSelection();
    try {
      ids = await invoke("search", { query: queryInput.value });
      if (ids.length === 0) {
        showMessage("No se encontraron correos.");
        moreBtn.style.display = "none";
        return;
      }
      showMessage(ids.length + " correos encontrados");
      await loadMore();
    } catch (error) {
      showMessage("Error: " + error, true);
    }
  });

  // Los atajos agregan su texto a la busqueda
  document.querySelector("#chips").addEventListener("click", (event) => {
    if (event.target.classList.contains("chip")) {
      queryInput.value = (queryInput.value + " " + event.target.textContent).trim();
      queryInput.focus();
    }
  });

  moreBtn.addEventListener("click", loadMore);

  rows.addEventListener("change", updateSelection);

  selectAll.addEventListener("change", () => {
    for (const box of rows.querySelectorAll("input")) {
      box.checked = selectAll.checked;
    }
    updateSelection();
  });

  // Mueve a la papelera solo los correos con la casilla marcada
  trashBtn.addEventListener("click", async () => {
    const boxes = [...rows.querySelectorAll("input:checked")];

    if (boxes.length === 0) {
      return;
    }

    if (!confirm("¿Mover " + boxes.length + " correo(s) a la papelera?")) {
      return;
    }

    trashBtn.disabled = true;
    showMessage("Moviendo a la papelera…");
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
      showMessage(
        "Movidos a la papelera: " + report.moved + ". Fallidos: " + report.failed.length + ".",
        report.failed.length > 0,
      );
    } catch (error) {
      showMessage("Error: " + error, true);
    }
    updateSelection();
  });
});