import { describe, it, expect } from "vitest";
import { getErrorMessage } from "./tauri";

describe("getErrorMessage", () => {
  it("returns string errors as-is", () => {
    expect(getErrorMessage("boom")).toBe("boom");
  });

  it("prefers the message of an Error instance", () => {
    expect(getErrorMessage(new Error("disk full"))).toBe("disk full");
  });

  it("falls back to the error name when message is empty", () => {
    const err = new Error("");
    err.name = "IoError";
    expect(getErrorMessage(err)).toBe("IoError");
  });

  it("formats a single-key AppError object as key: value", () => {
    expect(getErrorMessage({ NotFound: "/ext/missing.sub" })).toBe(
      "NotFound: /ext/missing.sub",
    );
  });

  it("joins multiple keys with a pipe", () => {
    expect(getErrorMessage({ IoError: "read failed", DbError: "locked" })).toBe(
      "IoError: read failed | DbError: locked",
    );
  });

  it("uses the key alone when its value is null or undefined", () => {
    expect(getErrorMessage({ PermissionDenied: null })).toBe("PermissionDenied: PermissionDenied");
  });

  it("falls back to Unknown error for non-object, non-string values", () => {
    expect(getErrorMessage(null)).toBe("Unknown error");
    expect(getErrorMessage(undefined)).toBe("Unknown error");
    expect(getErrorMessage(42)).toBe("Unknown error");
  });

  it("stringifies an object with no own enumerable keys", () => {
    expect(getErrorMessage({})).toBe("{}");
  });
});
