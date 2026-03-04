import { spawn } from "node:child_process";
import { resolve } from "node:path";

let tauriWdProcess;

export const config = {
  runner: "local",
  specs: ["./e2e/**/*.test.mjs"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        binary: resolve("./src-tauri/target/debug/ai-group-chat"),
      },
    },
  ],
  logLevel: "warn",
  waitforTimeout: 10000,
  connectionRetryTimeout: 30000,
  connectionRetryCount: 3,
  port: 4444,
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },

  onPrepare: function () {
    return new Promise((resolve, reject) => {
      const homedir = process.env.HOME || process.env.USERPROFILE;
      tauriWdProcess = spawn(`${homedir}/.cargo/bin/tauri-wd`, ["--port", "4444"], {
        stdio: ["ignore", "pipe", "pipe"],
      });

      tauriWdProcess.stderr.on("data", (data) => {
        const msg = data.toString();
        if (msg.includes("listening")) {
          resolve();
        }
      });

      // Fallback: resolve after 3s even if no "listening" message
      setTimeout(resolve, 3000);

      tauriWdProcess.on("error", reject);
    });
  },

  onComplete: function () {
    if (tauriWdProcess) {
      tauriWdProcess.kill();
    }
  },
};
