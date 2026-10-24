"use strict";

(() => {
    const panel = document.querySelector("#install-panel");
    const button = document.querySelector("#install-button");
    const help = document.querySelector("#install-help");
    const standalone = matchMedia("(display-mode: standalone)");
    let installPrompt = null;
    let dismissed = false;

    function installed() {
        return standalone.matches || navigator.standalone === true;
    }

    function updatePanel() {
        panel.hidden = installed() || dismissed;
    }

    function showMessage(message) {
        help.textContent = message;
        help.hidden = false;
    }

    document.querySelector("#install-close").addEventListener("click", () => {
        dismissed = true;
        updatePanel();
        document.querySelector("#glcanvas").focus();
    });

    window.addEventListener("beforeinstallprompt", (event) => {
        event.preventDefault();
        installPrompt = event;
        dismissed = false;
        updatePanel();
    });

    window.addEventListener("appinstalled", () => {
        installPrompt = null;
        panel.hidden = true;
    });
    standalone.addEventListener("change", updatePanel);

    button.addEventListener("click", async () => {
        if (installPrompt) {
            const prompt = installPrompt;
            installPrompt = null;
            button.disabled = true;
            try {
                await prompt.prompt();
                const choice = await prompt.userChoice;
                if (choice.outcome === "accepted") {
                    panel.hidden = true;
                } else {
                    showMessage("Instalación cancelada. Puedes instalar desde el menú del navegador.");
                }
            } catch (error) {
                console.error("Cannot show installation prompt", error);
                showMessage("No se pudo abrir la instalación. Prueba desde el menú del navegador.");
            } finally {
                button.disabled = false;
            }
        } else if (/iPad|iPhone|iPod/.test(navigator.userAgent)
            || (navigator.platform === "MacIntel" && navigator.maxTouchPoints > 1)) {
            showMessage("En Safari: toca Compartir y elige «Añadir a pantalla de inicio». Activa «Abrir como app» si aparece.");
        } else {
            showMessage("Abre el menú del navegador y busca «Instalar aplicación» o «Añadir a pantalla de inicio». Si no aparece, usa Chrome o Edge y espera a que termine la primera carga.");
        }
    });

    updatePanel();
    if ("serviceWorker" in navigator && window.isSecureContext) {
        navigator.serviceWorker.register("./sw.js").catch((error) => {
            console.error("Cannot register offline support", error);
            if (!installed()) {
                showMessage("No se pudo activar el modo sin conexión. Recarga con conexión a Internet.");
            }
        });
    }
})();
