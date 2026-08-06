import { describe, expect, it } from "vitest";
import { toFixError } from "./ipc";

describe("toFixError", () => {
  it("passes a real backend error straight through", () => {
    const backendError = {
      code: "rate_limited",
      message: "rate limit reached",
      remedy: "Wait about 30 seconds and try again.",
      retryable: true,
    };
    expect(toFixError(backendError)).toBe(backendError);
  });


  it("wraps anything that is not already a FixError", () => {
    const wrapped = toFixError("command not found");

    expect(wrapped.code).toBe("unexpected");
    expect(wrapped.message).toContain("command not found");
    expect(wrapped.remedy).not.toBe("");
    expect(wrapped.retryable).toBe(false);
  });

  it("survives null and undefined", () => {
    expect(toFixError(null).code).toBe("unexpected");
    expect(toFixError(undefined).code).toBe("unexpected");
  });

  it("does not mistake a partial object for a FixError", () => {

    expect(toFixError({ code: "x", message: "y" }).code).toBe("unexpected");
  });
});
