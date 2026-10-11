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
});