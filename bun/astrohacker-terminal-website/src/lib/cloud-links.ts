export type CloudEntry = "login" | "create";

export const FALLBACK_CLOUD_ORIGIN = "https://cloud.termsurf.com";

export function cloudOriginForHostname(hostname: string): string {
  const normalized = hostname.toLowerCase();
  if (normalized === "localhost" || normalized === "127.0.0.1") {
    return "http://127.0.0.1:3100";
  }
  if (normalized === "termsurf.test") return "https://cloud.termsurf.test";
  if (normalized === "termsurf2.test") return "https://cloud.termsurf2.test";
  if (normalized === "termsurf.com") return FALLBACK_CLOUD_ORIGIN;
  if (normalized === "termsurf2.com") return "https://cloud.termsurf2.com";
  return FALLBACK_CLOUD_ORIGIN;
}

export function cloudHrefForHostname(
  hostname: string,
  entry: CloudEntry,
): string {
  const origin = cloudOriginForHostname(hostname);
  return entry === "login" ? `${origin}/login` : `${origin}/`;
}
