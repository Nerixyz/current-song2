export function formatLocalUrl({
  path,
  host,
  port,
  protocol = 'http',
}: {
  path: string;
  host: string;
  port?: number;
  protocol?: string;
}): string {
  if (!port) {
    port = 48457;
  }

  if (process.env.APP_BASE) {
    return `${protocol}://${process.env.APP_BASE}${path}`;
  }

  return `${protocol}://${host}:${port}${path}`;
}
