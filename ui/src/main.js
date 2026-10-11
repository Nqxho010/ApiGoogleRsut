const { invoke } = window.__TAURI__.core;

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
  let ids = [];

  document.querySelector("#search-form").addEventListener("submit", async (event) => {
    event.preventDefault();
    result.textContent = "Buscando...";
    try {
      ids = await invoke("search", { query: document.querySelector("#query").value });
      result.textContent = "Se encontraron " + ids.length + " correos.";
    } catch (error) {
      result.textContent = "Error: " + error;
    }
  });
});