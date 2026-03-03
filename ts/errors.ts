export class SigocrError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "SigocrError";
  }
}

export class FileNotFoundError extends SigocrError {
  constructor(path: string, options?: ErrorOptions) {
    super(`File not found: ${path}`, options);
    this.name = "FileNotFoundError";
  }
}

export function toSigocrError(err: unknown): SigocrError | FileNotFoundError {
  if (err instanceof Error && err.message.startsWith("ENOENT: ")) {
    return new FileNotFoundError(err.message.slice("ENOENT: ".length), { cause: err });
  }
  return new SigocrError(err instanceof Error ? err.message : String(err), {
    cause: err instanceof Error ? err : undefined,
  });
}
