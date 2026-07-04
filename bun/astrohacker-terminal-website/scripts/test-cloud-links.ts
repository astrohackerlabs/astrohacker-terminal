import {
  FALLBACK_CLOUD_ORIGIN,
  cloudHrefForHostname,
  cloudOriginForHostname,
} from "../src/lib/cloud-links";

function assertEqual(actual: unknown, expected: unknown, label: string) {
  if (actual !== expected) {
    throw new Error(`${label}: expected ${expected}, got ${actual}`);
  }
}

assertEqual(
  cloudOriginForHostname("termsurf.test"),
  "https://cloud.termsurf.test",
  "primary test origin",
);
assertEqual(
  cloudOriginForHostname("termsurf2.test"),
  "https://cloud.termsurf2.test",
  "secondary test origin",
);
assertEqual(
  cloudOriginForHostname("termsurf.com"),
  "https://cloud.termsurf.com",
  "production origin",
);
assertEqual(
  cloudOriginForHostname("localhost"),
  "http://127.0.0.1:3100",
  "localhost origin",
);
assertEqual(
  cloudOriginForHostname("127.0.0.1"),
  "http://127.0.0.1:3100",
  "loopback origin",
);
assertEqual(
  cloudOriginForHostname("example.invalid"),
  FALLBACK_CLOUD_ORIGIN,
  "unknown fallback origin",
);
assertEqual(
  cloudHrefForHostname("termsurf.test", "login"),
  "https://cloud.termsurf.test/login",
  "primary login href",
);
assertEqual(
  cloudHrefForHostname("termsurf2.test", "create"),
  "https://cloud.termsurf2.test/",
  "secondary create href",
);
assertEqual(
  cloudHrefForHostname("localhost", "login"),
  "http://127.0.0.1:3100/login",
  "localhost login href",
);

console.log("ok cloud links: hostnames map to cloud login/create hrefs");
