/**
 * Image analysis utilities for detecting dungeon floor tiles from screenshots.
 *
 * Divides a screenshot into a grid of cells and classifies each cell as
 * mossy cobblestone, regular cobblestone, or unknown by comparing the
 * average colour of each cell against reference characteristics.
 *
 * The key insight: mossy cobblestone has a green tint (G channel elevated
 * relative to R and B), while regular cobblestone is neutral grey (R ≈ G ≈ B).
 * We use a normalised "green excess" metric that is invariant to brightness,
 * making detection robust across different lighting, gamma, and exposure.
 */

import { Tile } from "@/lib/types";

// ── Reference colours (computed from public/tiles/*.png) ──────────────
//
// mossy.png  → avg R≈104, G≈121, B≈104  (green tint, greenExcess ≈ 0.052)
// cobble.png → avg R≈123, G≈123, B≈123  (neutral grey, greenExcess ≈ 0.0)

export interface RGBColor {
  r: number;
  g: number;
  b: number;
}

export const REF_MOSSY: RGBColor = { r: 104, g: 121, b: 104 };
export const REF_COBBLE: RGBColor = { r: 123, g: 123, b: 123 };

/**
 * Compute the "green excess" of a colour: how much the green channel
 * exceeds the average of R and B, normalised by total brightness.
 * Positive → greenish (mossy), near-zero → neutral grey (cobble).
 * Returns 0 for black pixels to avoid division by zero.
 */
function greenExcess(c: RGBColor): number {
  const sum = c.r + c.g + c.b;
  if (sum < 15) return 0; // too dark to classify by hue
  // g / sum gives the green fraction (≈0.333 for grey)
  // subtract 1/3 so that neutral grey → 0, green tint → positive
  return c.g / sum - 1 / 3;
}

// Pre-computed reference green-excess values
const GE_MOSSY = greenExcess(REF_MOSSY);   // ≈ 0.052
const GE_COBBLE = greenExcess(REF_COBBLE); // ≈ 0.0
// Midpoint for classification
const GE_THRESHOLD = (GE_MOSSY + GE_COBBLE) / 2; // ≈ 0.026

// ── Analysis settings ─────────────────────────────────────────────────

export interface AnalysisSettings {
  /**
   * Minimum brightness (sum of R+G+B) to consider a cell classifiable.
   * Cells darker than this are marked unknown — they're too dark to
   * distinguish hue reliably.
   */
  minBrightness: number;
  /**
   * Maximum brightness to consider classifiable — cells brighter than
   * this are likely highlights / UI elements, not stone textures.
   */
  maxBrightness: number;
  /**
   * Saturation floor: minimum absolute green-excess magnitude needed
   * to classify with confidence.  Lower → more aggressive detection.
   */
  sensitivityThreshold: number;
}

export const DEFAULT_SETTINGS: AnalysisSettings = {
  minBrightness: 20,
  maxBrightness: 240,
  sensitivityThreshold: 0.008,
};

// ── Floor size inference from aspect ratio ────────────────────────────

/**
 * Given an image's width and height, try to infer the dungeon floor size
 * index from its aspect ratio.  Returns `null` if no clear match.
 *
 * FLOOR_SIZES indices: 0 = 9×9, 1 = 7×9, 2 = 9×7, 3 = 7×7
 */
export function inferFloorSizeIndex(
  imgWidth: number,
  imgHeight: number
): number | null {
  const ratio = imgWidth / imgHeight;

  // 9×7 → wider than tall (ratio ≈ 9/7 ≈ 1.286)
  if (Math.abs(ratio - 9 / 7) < 0.12) return 2;

  // 7×9 → taller than wide (ratio ≈ 7/9 ≈ 0.778)
  if (Math.abs(ratio - 7 / 9) < 0.12) return 1;

  return null;
}

// ── Core analysis ─────────────────────────────────────────────────────

/**
 * Load an image from a blob/file/URL into an ImageData.
 */
export function loadImageData(
  source: string | Blob
): Promise<{ imageData: ImageData; width: number; height: number }> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = img.naturalWidth;
      canvas.height = img.naturalHeight;
      const ctx = canvas.getContext("2d")!;
      ctx.drawImage(img, 0, 0);
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      resolve({ imageData, width: canvas.width, height: canvas.height });
    };
    img.onerror = () => reject(new Error("Failed to load image"));
    if (typeof source === "string") {
      img.src = source;
    } else {
      img.src = URL.createObjectURL(source);
    }
  });
}

/**
 * Compute the average RGB colour of a rectangular region within ImageData.
 * Skips fully transparent pixels.
 */
function averageColor(
  data: ImageData,
  x0: number,
  y0: number,
  w: number,
  h: number
): RGBColor {
  const d = data.data;
  const stride = data.width * 4;
  let rSum = 0;
  let gSum = 0;
  let bSum = 0;
  let count = 0;

  const xEnd = Math.min(x0 + w, data.width);
  const yEnd = Math.min(y0 + h, data.height);

  for (let y = y0; y < yEnd; y++) {
    for (let x = x0; x < xEnd; x++) {
      const i = y * stride + x * 4;
      if (d[i + 3] < 128) continue; // skip transparent
      rSum += d[i];
      gSum += d[i + 1];
      bSum += d[i + 2];
      count++;
    }
  }

  if (count === 0) return { r: 0, g: 0, b: 0 };
  return { r: rSum / count, g: gSum / count, b: bSum / count };
}

/**
 * Euclidean distance between two RGB colours.
 */
function rgbDistance(a: RGBColor, b: RGBColor): number {
  const dr = a.r - b.r;
  const dg = a.g - b.g;
  const db = a.b - b.b;
  return Math.sqrt(dr * dr + dg * dg + db * db);
}

export interface CellAnalysis {
  tile: Tile;
  avgColor: RGBColor;
  distMossy: number;
  distCobble: number;
  /** Green-excess value: positive = green tint (mossy), ~0 = neutral (cobble) */
  ge: number;
  confidence: "high" | "medium" | "low";
}

/**
 * Analyse an image and classify each cell in the grid.
 *
 * Classification uses a brightness-invariant "green excess" metric:
 *   ge = G/(R+G+B) − 1/3
 * Mossy cobblestone has a positive green excess (green tint), while
 * regular cobblestone is near zero (neutral grey).  This is robust
 * across different lighting levels, gamma, and exposure.
 *
 * @param imageData  Raw pixel data from the image.
 * @param cols       Number of grid columns (X dimension).
 * @param rows       Number of grid rows (Z dimension).
 * @param settings   Analysis settings.
 * @returns          2D array [row][col] of CellAnalysis results.
 */
export function analyseImage(
  imageData: ImageData,
  cols: number,
  rows: number,
  settings: AnalysisSettings
): CellAnalysis[][] {
  const cellW = imageData.width / cols;
  const cellH = imageData.height / rows;

  // Shrink the sampling region to the inner 90% to avoid alignment issues
  const insetX = cellW * 0.05;
  const insetY = cellH * 0.05;
  const sampleW = cellW * 0.9;
  const sampleH = cellH * 0.9;

  const result: CellAnalysis[][] = [];

  for (let row = 0; row < rows; row++) {
    const rowCells: CellAnalysis[] = [];
    for (let col = 0; col < cols; col++) {
      const x0 = Math.round(col * cellW + insetX);
      const y0 = Math.round(row * cellH + insetY);
      const w = Math.round(sampleW);
      const h = Math.round(sampleH);

      const avg = averageColor(imageData, x0, y0, w, h);
      const dMossy = rgbDistance(avg, REF_MOSSY);
      const dCobble = rgbDistance(avg, REF_COBBLE);

      // Brightness check
      const brightness = (avg.r + avg.g + avg.b) / 3;
      const ge = greenExcess(avg);

      let tile: Tile;
      let confidence: "high" | "medium" | "low";

      if (brightness < settings.minBrightness || brightness > settings.maxBrightness) {
        // Too dark or too bright to classify reliably
        tile = Tile.Unknown;
        confidence = "low";
      } else {
        // Classify by green excess (hue-based, brightness-invariant)
        const distFromThreshold = ge - GE_THRESHOLD;

        if (Math.abs(distFromThreshold) < settings.sensitivityThreshold) {
          // Too ambiguous — right on the boundary
          tile = distFromThreshold >= 0 ? Tile.Mossy : Tile.Cobble;
          confidence = "low";
        } else if (distFromThreshold > 0) {
          tile = Tile.Mossy;
          confidence = distFromThreshold > GE_THRESHOLD ? "high" : "medium";
        } else {
          tile = Tile.Cobble;
          confidence = Math.abs(distFromThreshold) > GE_THRESHOLD ? "high" : "medium";
        }
      }

      rowCells.push({ tile, avgColor: avg, distMossy: dMossy, distCobble: dCobble, ge, confidence });
    }
    result.push(rowCells);
  }

  return result;
}

/**
 * Convert analysis results into a Tile[][] suitable for the floor grid.
 * Places results into the correct sub-region of a 9×9 grid based on
 * the floor size index.
 */
export function analysisToFloor(
  analysis: CellAnalysis[][],
  cols: number,
  rows: number,
  xMin: number,
  zMin: number
): Tile[][] {
  // Start with unknown-solid fill
  const grid: Tile[][] = Array.from({ length: 9 }, () =>
    new Array<Tile>(9).fill(Tile.UnknownSolid)
  );

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      grid[zMin + row][xMin + col] = analysis[row][col].tile;
    }
  }

  return grid;
}
