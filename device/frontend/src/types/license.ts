export interface LicenseStatus {
  licensed: boolean;
  machine_id: string | null;
  license_id: string | null;
  tier: string | null;
  features: string[];
  expires_at: string | null;
  message: string | null;
}
