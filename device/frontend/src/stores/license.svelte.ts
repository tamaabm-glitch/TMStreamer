import { api, message } from "@/api/client";
import type { LicenseStatus } from "@/types/license";

class LicenseStore {
  loaded = $state(false);
  loading = $state(false);
  activating = $state(false);
  status = $state<LicenseStatus | null>(null);
  error = $state<string | null>(null);

  licensed = $derived(this.status?.licensed === true);
  canStream = $derived(this.licensed && this.status?.features.includes("streaming") === true);
  canRecord = $derived(this.licensed && this.status?.features.includes("recording") === true);
  machineId = $derived(this.status?.machine_id ?? null);

  async load() {
    if (this.loading) return;
    this.loading = true;
    try {
      this.status = await api.license();
      this.error = null;
    } catch (error) {
      this.error = message(error);
    } finally {
      this.loaded = true;
      this.loading = false;
    }
  }

  async activate(file: File) {
    if (this.activating) return;
    this.activating = true;
    this.error = null;
    try {
      this.status = await api.activateLicense(file);
      if (!this.status.licensed) {
        throw new Error(this.status.message ?? "Lisensi belum dapat diaktifkan.");
      }
    } catch (error) {
      this.error = message(error);
      throw error;
    } finally {
      this.activating = false;
    }
  }
}

export const license = new LicenseStore();
