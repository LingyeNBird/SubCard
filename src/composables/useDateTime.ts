function displayDateTime(
  value: string | null | undefined,
  fallback: string,
): string {
  if (!value) return fallback;
  const instant = new Date(value);
  if (Number.isNaN(instant.getTime())) return fallback;
  return instant.toLocaleString("zh-CN", { hour12: false });
}

export function useDateTime(fallback = "—") {
  return (value: string | null | undefined) => displayDateTime(value, fallback);
}
