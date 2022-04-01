export function formatLocalUrl(path: string, port = 48457, protocol = 'http'): string {
  if (process.env.APP_BASE) {
    return `${protocol}://${process.env.APP_BASE}${path}`;
  } else {
    return `${protocol}://127.0.0.1:${port}${path}`;
  }
}
