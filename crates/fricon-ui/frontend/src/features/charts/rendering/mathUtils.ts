/** Shared math utilities for pixel ↔ data-space conversions. */

export function invertZoomedLinearRange(
  value: number,
  zoomOffset: number,
  zoomScale: number,
  domainStart: number,
  domainEnd: number,
  rangeStart: number,
  rangeEnd: number,
): number {
  return invertLinearRange(
    (value - zoomOffset) / zoomScale,
    domainStart,
    domainEnd,
    rangeStart,
    rangeEnd,
  );
}

export function invertLinearRange(
  value: number,
  domainStart: number,
  domainEnd: number,
  rangeStart: number,
  rangeEnd: number,
): number {
  if (rangeEnd === rangeStart) return domainStart;
  return (
    domainStart +
    ((value - rangeStart) / (rangeEnd - rangeStart)) * (domainEnd - domainStart)
  );
}

export function projectLinearRange(
  value: number,
  domainStart: number,
  domainEnd: number,
  rangeStart: number,
  rangeEnd: number,
): number {
  if (domainEnd === domainStart) {
    return (rangeStart + rangeEnd) / 2;
  }
  return (
    rangeStart +
    ((value - domainStart) / (domainEnd - domainStart)) *
      (rangeEnd - rangeStart)
  );
}
