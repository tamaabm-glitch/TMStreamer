import { snackbar } from "@/stores/snackbar.svelte";
import { readMigrated } from "@/lib/storage";

const KEY = "tmstreamer.developer";
const LEGACY_KEY = "ystreamer.developer";
const TAPS = 7;
const HINT_FROM = 3;
const TAP_GAP_MS = 1500;

function load(): boolean {
  try {
    return readMigrated(KEY, LEGACY_KEY) === "1";
  } catch {
    return false;
  }
}

function save(on: boolean) {
  try {
    if (on) localStorage.setItem(KEY, "1");
    else localStorage.removeItem(KEY);
  } catch {}
}

class Developer {
  enabled = $state(load());
  #taps = 0;
  #last = 0;

  tap() {
    if (this.enabled) {
      snackbar.show("Developer mode is already on");
      return;
    }
    const now = Date.now();
    this.#taps = now - this.#last > TAP_GAP_MS ? 1 : this.#taps + 1;
    this.#last = now;

    const left = TAPS - this.#taps;
    if (left === 0) {
      this.#taps = 0;
      this.set(true);
      snackbar.show(
        "Developer mode on. Find the tools under System → Developer.",
      );
    } else if (this.#taps >= HINT_FROM) {
      snackbar.show(
        `${left} ${left === 1 ? "tap" : "taps"} away from developer mode`,
      );
    }
  }

  set(on: boolean) {
    this.enabled = on;
    save(on);
  }
}

export const developer = new Developer();
