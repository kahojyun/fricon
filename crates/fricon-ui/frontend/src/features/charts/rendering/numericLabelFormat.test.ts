import { describe, expect, it } from "vitest";
import { formatNumericLabel } from "./numericLabelFormat";

describe("formatNumericLabel", () => {
  it("formats zero without notation changes", () => {
    expect(formatNumericLabel(0, { mode: "si", significantDigits: 4 })).toBe(
      "0",
    );
    expect(
      formatNumericLabel(-0, { mode: "scientific", significantDigits: 4 }),
    ).toBe("0");
  });

  it("formats SI-prefixed values with trimmed trailing zeroes", () => {
    expect(formatNumericLabel(1234, { mode: "si", significantDigits: 4 })).toBe(
      "1.234k",
    );
    expect(
      formatNumericLabel(0.00000123, { mode: "si", significantDigits: 4 }),
    ).toBe("1.23µ");
  });

  it("formats scientific values with significant digits", () => {
    expect(
      formatNumericLabel(-0.00012345, {
        mode: "scientific",
        significantDigits: 4,
      }),
    ).toBe("-1.234e-4");
  });

  it("formats decimal values without scientific notation", () => {
    expect(
      formatNumericLabel(12345, { mode: "decimal", significantDigits: 4 }),
    ).toBe("12350");
    expect(
      formatNumericLabel(-12345, { mode: "decimal", significantDigits: 4 }),
    ).toBe("-12350");
    expect(
      formatNumericLabel(12.34, { mode: "decimal", significantDigits: 4 }),
    ).toBe("12.34");
  });

  it("switches auto mode between decimal and scientific thresholds", () => {
    expect(
      formatNumericLabel(0.001234, { mode: "auto", significantDigits: 4 }),
    ).toBe("0.001234");
    expect(
      formatNumericLabel(0.0001234, { mode: "auto", significantDigits: 4 }),
    ).toBe("1.234e-4");
    expect(
      formatNumericLabel(10000, { mode: "auto", significantDigits: 4 }),
    ).toBe("1e+4");
  });

  it("clamps significant digits when options fall out of range", () => {
    expect(
      formatNumericLabel(1234, { mode: "si", significantDigits: 20 }),
    ).toBe("1.234k");
    expect(
      formatNumericLabel(12.345, { mode: "decimal", significantDigits: 1 }),
    ).toBe("12");
  });
});
