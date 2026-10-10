export function redact(text: string): string {
 return text.replace(/\b(Bearer|Basic)\s+[A-Za-z0-9_./+=-]+/gi,"$1 [已脱敏]")
  .replace(/((?:[a-z_]*token|authorization|cookie|password|secret|credential|api[-_]?key)["']?\s*[:=]\s*)("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|[^\s,"'&}\]]+)/gi,"$1[已脱敏]")
  .replace(/\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b/g,"[已脱敏]");
}
