export const Config = {
  serverUrl:
    typeof window !== "undefined" && window.location.href.includes("localhost")
      ? "http://localhost:3000"
      : typeof window !== "undefined"
        ? `${window.location.protocol}//${window.location.hostname}`
        : "http://localhost:3000",
}
