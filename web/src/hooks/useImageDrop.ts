"use client";

import { useEffect, useCallback, useRef } from "react";
import {
  loadImageData,
  analyseImage,
  analysisToFloor,
  DEFAULT_SETTINGS,
} from "@/lib/image-analysis";
import { Tile, FLOOR_SIZES } from "@/lib/types";

interface UseImageDropOptions {
  /** Current floor size index — used when no dialog is open. */
  floorSizeIndex: number;
  /** Whether the picture-import dialog is currently open (skip global handling). */
  dialogOpen: boolean;
  /** Callback to apply the detected floor pattern. */
  onApply: (floor: Tile[][], sizeIndex: number) => void;
}

/**
 * Hook that listens for paste and drag-drop events on the whole page.
 * When an image is pasted/dropped while the dialog is NOT open, it runs
 * the analysis with the current floor size and applies the result directly.
 */
export function useImageDrop({
  floorSizeIndex,
  dialogOpen,
  onApply,
}: UseImageDropOptions) {
  // Keep latest values in refs so the event handlers always see current state
  const sizeRef = useRef(floorSizeIndex);
  const dialogRef = useRef(dialogOpen);
  const applyRef = useRef(onApply);

  useEffect(() => { sizeRef.current = floorSizeIndex; }, [floorSizeIndex]);
  useEffect(() => { dialogRef.current = dialogOpen; }, [dialogOpen]);
  useEffect(() => { applyRef.current = onApply; }, [onApply]);

  const processImage = useCallback(async (blob: Blob) => {
    if (dialogRef.current) return; // dialog handles its own images

    const url = URL.createObjectURL(blob);
    try {
      const { imageData } = await loadImageData(url);
      const fs = FLOOR_SIZES[sizeRef.current];
      const cols = fs.xMax - fs.xMin;
      const rows = fs.zMax - fs.zMin;
      const analysis = analyseImage(imageData, cols, rows, DEFAULT_SETTINGS);
      const floor = analysisToFloor(analysis, cols, rows, fs.xMin, fs.zMin);
      applyRef.current(floor, sizeRef.current);
    } finally {
      URL.revokeObjectURL(url);
    }
  }, []);

  // ── Global paste ──
  useEffect(() => {
    const handler = (e: ClipboardEvent) => {
      if (dialogRef.current) return; // dialog will handle it
      const items = e.clipboardData?.items;
      if (!items) return;
      for (const item of items) {
        if (item.type.startsWith("image/")) {
          const blob = item.getAsFile();
          if (blob) {
            e.preventDefault();
            processImage(blob);
            return;
          }
        }
      }
    };
    document.addEventListener("paste", handler);
    return () => document.removeEventListener("paste", handler);
  }, [processImage]);

  // ── Global drag & drop ──
  useEffect(() => {
    const handleDragOver = (e: DragEvent) => {
      if (dialogRef.current) return;
      // Only accept files
      if (e.dataTransfer?.types.includes("Files")) {
        e.preventDefault();
      }
    };

    const handleDrop = (e: DragEvent) => {
      if (dialogRef.current) return;
      const file = e.dataTransfer?.files?.[0];
      if (file?.type.startsWith("image/")) {
        e.preventDefault();
        processImage(file);
      }
    };

    document.addEventListener("dragover", handleDragOver);
    document.addEventListener("drop", handleDrop);
    return () => {
      document.removeEventListener("dragover", handleDragOver);
      document.removeEventListener("drop", handleDrop);
    };
  }, [processImage]);
}
