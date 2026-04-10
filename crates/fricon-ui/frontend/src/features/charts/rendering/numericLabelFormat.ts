import { format as d3Format } from "d3-format";
import type { NumericLabelFormatOptions } from "@/shared/lib/chartTypes";

export const MIN_SIGNIFICANT_DIGITS = 2;
export const MAX_SIGNIFICANT_DIGITS = 8;

export const DEFAULT_NUMERIC_LABEL_FORMAT: NumericLabelFormatOptions = {
  mode: "si",
  significantDigits: 4,
};

export function clampSignificantDigits(value: number): number {
  const normalized = Number.isFinite(value)
    ? Math.trunc(value)
    : DEFAULT_NUMERIC_LABEL_FORMAT.significantDigits;
  return Math.min(
    MAX_SIGNIFICANT_DIGITS,
    Math.max(MIN_SIGNIFICANT_DIGITS, normalized),
  );
}

export function formatNumericLabel(
  value: number,
  options: NumericLabelFormatOptions,
): string {
  if (!Number.isFinite(value)) {
    return String(value);
  }
  if (Object.is(value, -0) || value === 0) {
    return "0";
  }

  const significantDigits = clampSignificantDigits(options.significantDigits);
  switch (options.mode) {
    case "si":
      return normalizeFormattedNumber(
        d3Format(`.${significantDigits}~s`)(value),
      );
    case "scientific":
      return normalizeFormattedNumber(
        d3Format(`.${Math.max(significantDigits - 1, 0)}~e`)(value),
      );
    case "decimal":
      return formatDecimal(value, significantDigits);
    case "auto":
      return formatAuto(value, significantDigits);
    default:
      return assertNever(options.mode);
  }
}

export function formatAxisTickLabel(
  value: string | number,
  options: NumericLabelFormatOptions,
): string {
  return typeof value === "number" ? formatNumericLabel(value, options) : value;
}

function formatAuto(value: number, significantDigits: number): string {
  const magnitude = Math.abs(value);
  if (magnitude >= 1e-3 && magnitude < 1e4) {
    return formatDecimal(value, significantDigits);
  }
  return normalizeFormattedNumber(
    d3Format(`.${Math.max(significantDigits - 1, 0)}~e`)(value),
  );
}

function formatDecimal(value: number, significantDigits: number): string {
  const magnitude = Math.abs(value);
  const exponent = Math.floor(Math.log10(magnitude));
  const decimals = significantDigits - exponent - 1;

  if (decimals >= 0) {
    return normalizeFormattedNumber(d3Format(`.${decimals}~f`)(value));
  }

  const roundingFactor = 10 ** -decimals;
  const scaled = value / roundingFactor;
  const rounded =
    Math.sign(scaled) * Math.round(Math.abs(scaled)) * roundingFactor;
  return normalizeFormattedNumber(d3Format(".0f")(rounded));
}

function assertNever(value: never): never {
  throw new Error(`Unsupported numeric label format mode: ${String(value)}`);
}

function normalizeFormattedNumber(value: string): string {
  return value.replace(/\u2212/g, "-");
}
