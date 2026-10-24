"use strict";

miniquad_add_plugin({
    name: "arcade_scores",
    version: 1,
    register_plugin(imports) {
        const key = "tap-away-arcade.highscores.v1";
        const encoder = new TextEncoder();
        const decoder = new TextDecoder("utf-8", { fatal: true });

        imports.env.arcade_scores_read = function (pointer, capacity) {
            try {
                const stored = localStorage.getItem(key);
                if (stored === null) return 0;
                const bytes = encoder.encode(stored);
                if (bytes.length > capacity) {
                    console.error("Highscore data exceeds storage buffer");
                    return -1;
                }
                new Uint8Array(wasm_memory.buffer, pointer, bytes.length).set(bytes);
                return bytes.length;
            } catch (error) {
                console.error("Cannot read highscores", error);
                return -1;
            }
        };

        imports.env.arcade_scores_write = function (pointer, length) {
            try {
                const data = decoder.decode(new Uint8Array(wasm_memory.buffer, pointer, length));
                localStorage.setItem(key, data);
                return 0;
            } catch (error) {
                console.error("Cannot save highscores", error);
                return -1;
            }
        };
    }
});
