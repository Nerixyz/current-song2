export function formatLocalUrl(path: string, protocol = 'http'): string {
  if (process.env.APP_BASE) {
    return `${protocol}://${process.env.APP_BASE}${path}`;
  } else {
    return `${protocol}://127.0.0.1:48457${path}`;
  }
}
