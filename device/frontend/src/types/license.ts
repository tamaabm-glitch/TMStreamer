export interface LicenseStatus {
  product: string;
  license_system: string;
  activated: boolean;
  status: string;
  message: string;
  tier: string | null;
  license_id: string | null;
  expires_at: string | null;
  features: string[];
  key_id: string | null;
}
