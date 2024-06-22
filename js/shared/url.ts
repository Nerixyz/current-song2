export function formatLocalUrl({
  path,
  host,
  port = 48457,
  protocol = 'http',
}: {
  path: string;
  host: string;
  port?: number;
  protocol?: string;
}): string {
  if (process.env.APP_BASE) {
    return `${protocol}://${process.env.APP_BASE}${path}`;
  } else {
    return `${protocol}://${host}:${port}${path}`;
  }
}
