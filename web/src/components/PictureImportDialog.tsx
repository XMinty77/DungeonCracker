"use client";

import { useState, useCallback, useRef, useEffect, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Image from "next/image";
import {
  X,
  Upload,
  ClipboardPaste,
  ImageIcon,
  Settings2,
  Check,
  AlertTriangle,
  RotateCcw,
} from "lucide-react";
import {
  Tile,
  TILE_IMAGES,
  TILE_NAMES,
  FLOOR_SIZES,
} from "@/lib/types";
import {
  loadImageData,
  analyseImage,
  analysisToFloor,
  inferFloorSizeIndex,
  DEFAULT_SETTINGS,
  type AnalysisSettings,
} from "@/lib/image-analysis";

// ── Props ─────────────────────────────────────────────────────────────

interface PictureImportDialogProps {
  open: boolean;
  floorSizeIndex: number;
  onClose: () => void;
  onApply: (floor: Tile[][], sizeIndex: number) => void;
}

// ── Component ─────────────────────────────────────────────────────────

export function PictureImportDialog({
  open,
  floorSizeIndex: externalSizeIndex,
  onClose,
  onApply,
}: PictureImportDialogProps) {
  // ── Image state ──
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [imgSize, setImgSize] = useState<{ w: number; h: number } | null>(null);
  const [imageData, setImageData] = useState<ImageData | null>(null);

  // ── Settings ──
  // null = follow external prop; number = user/inferred override
  const [sizeOverride, setSizeOverride] = useState<number | null>(null);
  const sizeIndex = sizeOverride ?? externalSizeIndex;
  const [settings, setSettings] = useState<AnalysisSettings>(DEFAULT_SETTINGS);
  const [showSettings, setShowSettings] = useState(false);

  // ── Drag state ──
  const [isDragging, setIsDragging] = useState(false);

  const fileInputRef = useRef<HTMLInputElement>(null);

  // Clean up object URL on unmount or new image
  useEffect(() => {
    return () => {
      if (imageUrl?.startsWith("blob:")) URL.revokeObjectURL(imageUrl);
    };
  }, [imageUrl]);

  const fs = FLOOR_SIZES[sizeIndex];
  const cols = fs.xMax - fs.xMin;
  const rows = fs.zMax - fs.zMin;

  // ── Derive analysis from current state (pure computation) ──
  const analysis = useMemo(() => {
    if (!imageData) return null;
    const currentFs = FLOOR_SIZES[sizeIndex];
    const c = currentFs.xMax - currentFs.xMin;
    const r = currentFs.zMax - currentFs.zMin;
    return analyseImage(imageData, c, r, settings);
  }, [imageData, sizeIndex, settings]);

  // ── Load image from file / blob ──
  const handleImage = useCallback(
    async (source: File | Blob) => {
      const url = URL.createObjectURL(source);
      setImageUrl(url);
      try {
        const { imageData: imgData, width, height } = await loadImageData(url);
        setImageData(imgData);
        setImgSize({ w: width, h: height });

        // Try to infer floor size from aspect ratio
        const inferred = inferFloorSizeIndex(width, height);
        if (inferred !== null) {
          setSizeOverride(inferred);
        }
      } catch {
        setImageData(null);
        setImgSize(null);
      }
    },
    []
  );

  // ── File input handler ──
  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) handleImage(file);
    },
    [handleImage]
  );

  // ── Paste handler (inside dialog) ──
  const handlePaste = useCallback(
    (e: React.ClipboardEvent | ClipboardEvent) => {
      const items = (e as ClipboardEvent).clipboardData?.items ?? (e as React.ClipboardEvent).clipboardData?.items;
      if (!items) return;
      for (const item of items) {
        if (item.type.startsWith("image/")) {
          const blob = item.getAsFile();
          if (blob) {
            e.preventDefault();
            handleImage(blob);
            return;
          }
        }
      }
    },
    [handleImage]
  );

  // Listen for paste while dialog is open
  useEffect(() => {
    if (!open) return;
    const handler = (e: ClipboardEvent) => handlePaste(e);
    document.addEventListener("paste", handler);
    return () => document.removeEventListener("paste", handler);
  }, [open, handlePaste]);

  // ── Drag & drop handlers ──
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);
      const file = e.dataTransfer.files?.[0];
      if (file?.type.startsWith("image/")) {
        handleImage(file);
      }
    },
    [handleImage]
  );

  // ── Apply results ──
  const handleApply = useCallback(() => {
    if (!analysis) return;
    const floor = analysisToFloor(analysis, cols, rows, fs.xMin, fs.zMin);
    onApply(floor, sizeIndex);
    onClose();
  }, [analysis, cols, rows, fs, sizeIndex, onApply, onClose]);

  // ── Reset ──
  const handleReset = useCallback(() => {
    if (imageUrl?.startsWith("blob:")) URL.revokeObjectURL(imageUrl);
    setImageUrl(null);
    setImageData(null);
    setImgSize(null);
    setSizeOverride(null);
  }, [imageUrl]);

  // ── Close & cleanup ──
  const handleClose = useCallback(() => {
    handleReset();
    onClose();
  }, [handleReset, onClose]);

  // ── Count stats ──
  const stats = useMemo(() => {
    if (!analysis) return null;
    let mossy = 0, cobble = 0, unknown = 0;
    let highConf = 0, medConf = 0, lowConf = 0;
    for (const row of analysis) {
      for (const cell of row) {
        if (cell.tile === Tile.Mossy) mossy++;
        else if (cell.tile === Tile.Cobble) cobble++;
        else unknown++;
        if (cell.confidence === "high") highConf++;
        else if (cell.confidence === "medium") medConf++;
        else lowConf++;
      }
    }
    return { mossy, cobble, unknown, highConf, medConf, lowConf, total: mossy + cobble + unknown };
  }, [analysis]);

  if (!open) return null;

  return (
    <AnimatePresence>
      {open && (
        <>
          {/* Backdrop */}
          <motion.div
            key="backdrop"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-[100] bg-black/70 backdrop-blur-sm"
            onClick={handleClose}
          />

          {/* Dialog */}
          <motion.div
            key="dialog"
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            transition={{ duration: 0.2, ease: "easeOut" }}
            className="fixed inset-0 z-[101] flex items-center justify-center p-4"
            onClick={(e) => {
              if (e.target === e.currentTarget) handleClose();
            }}
          >
            <div className="mc-panel w-full max-w-2xl max-h-[90vh] overflow-y-auto">
              {/* Header */}
              <div className="flex items-center justify-between p-4 border-b border-mc-border">
                <div className="flex items-center gap-2">
                  <ImageIcon className="w-4 h-4 text-mc-green-text" />
                  <h2 className="font-minecraft text-sm text-mc-text-highlight tracking-wide">
                    Import from Picture
                  </h2>
                </div>
                <button
                  onClick={handleClose}
                  className="p-1 text-mc-text-dim hover:text-mc-text transition-colors"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>

              {/* Body */}
              <div className="p-4 space-y-4">
                {/* ── Upload zone ── */}
                {!imageUrl && (
                  <div
                    onDragOver={handleDragOver}
                    onDragLeave={handleDragLeave}
                    onDrop={handleDrop}
                    onClick={() => fileInputRef.current?.click()}
                    className={`
                      flex flex-col items-center justify-center gap-3 p-8
                      border-2 border-dashed rounded cursor-pointer
                      transition-colors duration-200
                      ${isDragging
                        ? "border-mc-green bg-mc-green/10"
                        : "border-mc-border hover:border-mc-green hover:bg-mc-bg-darker/50"
                      }
                    `}
                  >
                    <Upload className="w-8 h-8 text-mc-text-dim" />
                    <div className="text-center">
                      <p className="text-sm text-mc-text font-semibold">
                        Drop an image here, or click to browse
                      </p>
                      <p className="text-xs text-mc-text-dim mt-1">
                        You can also paste from clipboard (Ctrl+V)
                      </p>
                    </div>
                    <div className="flex gap-2 mt-2">
                      <span className="inline-flex items-center gap-1 text-[10px] text-mc-text-dim bg-mc-bg-darker px-2 py-1 border border-mc-border">
                        <Upload className="w-3 h-3" />
                        Upload
                      </span>
                      <span className="inline-flex items-center gap-1 text-[10px] text-mc-text-dim bg-mc-bg-darker px-2 py-1 border border-mc-border">
                        <ClipboardPaste className="w-3 h-3" />
                        Ctrl+V
                      </span>
                    </div>
                    <input
                      ref={fileInputRef}
                      type="file"
                      accept="image/*"
                      onChange={handleFileSelect}
                      className="hidden"
                    />
                  </div>
                )}

                {/* ── Image preview + grid overlay ── */}
                {imageUrl && (
                  <>
                    <div className="relative">
                      {/* Image with overlay */}
                      <div className="relative w-full" style={{ aspectRatio: imgSize ? `${imgSize.w}/${imgSize.h}` : "1/1" }}>
                        {/* eslint-disable-next-line @next/next/no-img-element */}
                        <img
                          src={imageUrl}
                          alt="Uploaded dungeon screenshot"
                          className="w-full h-full object-contain"
                          draggable={false}
                        />

                        {/* Grid overlay */}
                        <div className="absolute inset-0">
                          {/* Vertical lines */}
                          {Array.from({ length: cols - 1 }, (_, i) => (
                            <div
                              key={`v-${i}`}
                              className="absolute top-0 bottom-0 w-px bg-mc-green/50"
                              style={{ left: `${((i + 1) / cols) * 100}%` }}
                            />
                          ))}
                          {/* Horizontal lines */}
                          {Array.from({ length: rows - 1 }, (_, i) => (
                            <div
                              key={`h-${i}`}
                              className="absolute left-0 right-0 h-px bg-mc-green/50"
                              style={{ top: `${((i + 1) / rows) * 100}%` }}
                            />
                          ))}

                          {/* Cell overlays showing detected tile */}
                          {analysis &&
                            analysis.map((row, rIdx) =>
                              row.map((cell, cIdx) => {
                                const cellLeft = (cIdx / cols) * 100;
                                const cellTop = (rIdx / rows) * 100;
                                const cellWidth = (1 / cols) * 100;
                                const cellHeight = (1 / rows) * 100;

                                const bg =
                                  cell.tile === Tile.Mossy
                                    ? "rgba(76, 153, 76, 0.25)"
                                    : cell.tile === Tile.Cobble
                                      ? "rgba(160, 160, 160, 0.25)"
                                      : "rgba(255, 85, 85, 0.2)";

                                const borderColor =
                                  cell.confidence === "high"
                                    ? "rgba(82, 165, 53, 0.6)"
                                    : cell.confidence === "medium"
                                      ? "rgba(255, 196, 43, 0.6)"
                                      : "rgba(255, 85, 85, 0.6)";

                                return (
                                  <div
                                    key={`cell-${rIdx}-${cIdx}`}
                                    className="absolute flex items-center justify-center"
                                    style={{
                                      left: `${cellLeft}%`,
                                      top: `${cellTop}%`,
                                      width: `${cellWidth}%`,
                                      height: `${cellHeight}%`,
                                      background: bg,
                                      border: `1px solid ${borderColor}`,
                                    }}
                                    title={`${TILE_NAMES[cell.tile]} (GE:${(cell.ge * 1000).toFixed(1)} RGB:${cell.avgColor.r.toFixed(0)},${cell.avgColor.g.toFixed(0)},${cell.avgColor.b.toFixed(0)})`}
                                  >
                                    <span className="text-[10px] font-bold text-white drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]">
                                      {cell.tile === Tile.Mossy
                                        ? "M"
                                        : cell.tile === Tile.Cobble
                                          ? "C"
                                          : "?"}
                                    </span>
                                  </div>
                                );
                              })
                            )}
                        </div>
                      </div>

                      {/* Reset image button */}
                      <button
                        onClick={handleReset}
                        className="absolute top-2 right-2 p-1.5 bg-mc-bg/80 border border-mc-border text-mc-text-dim hover:text-mc-text hover:bg-mc-bg transition-colors"
                        title="Remove image"
                      >
                        <RotateCcw className="w-3.5 h-3.5" />
                      </button>
                    </div>

                    {/* ── Controls ── */}
                    <div className="space-y-3">
                      {/* Floor size selector */}
                      <div className="flex flex-wrap items-center gap-3">
                        <span className="text-xs text-mc-text-dim font-semibold uppercase tracking-wider">
                          Floor size:
                        </span>
                        <div className="flex items-center gap-2">
                          {/* X dimension */}
                          <div className="flex">
                            {([9, 7] as const).map((val) => {
                              const currentFs = FLOOR_SIZES[sizeIndex];
                              const isActive = val === (currentFs.xMax - currentFs.xMin);
                              return (
                                <button
                                  key={`x-${val}`}
                                  onClick={() => {
                                    const currentFs2 = FLOOR_SIZES[sizeIndex];
                                    const zDim = currentFs2.zMax - currentFs2.zMin;
                                    const idx = (val === 7 ? 1 : 0) + (zDim === 7 ? 2 : 0);
                                    setSizeOverride(idx);
                                  }}
                                  className={`h-7 px-2.5 text-xs font-semibold transition-colors duration-200 border-y-2 border-x first:border-l-2 last:border-r-2 ${
                                    isActive
                                      ? "bg-mc-green text-white border-mc-green border-t-mc-green-light border-b-mc-green-dark"
                                      : "bg-mc-bg-darker text-mc-text-dim border-mc-border hover:text-mc-text hover:bg-mc-tab-active"
                                  }`}
                                >
                                  {val}
                                </button>
                              );
                            })}
                          </div>
                          <span className="text-xs text-mc-text-dim font-bold select-none">×</span>
                          {/* Z dimension */}
                          <div className="flex">
                            {([9, 7] as const).map((val) => {
                              const currentFs = FLOOR_SIZES[sizeIndex];
                              const isActive = val === (currentFs.zMax - currentFs.zMin);
                              return (
                                <button
                                  key={`z-${val}`}
                                  onClick={() => {
                                    const currentFs2 = FLOOR_SIZES[sizeIndex];
                                    const xDim = currentFs2.xMax - currentFs2.xMin;
                                    const idx = (xDim === 7 ? 1 : 0) + (val === 7 ? 2 : 0);
                                    setSizeOverride(idx);
                                  }}
                                  className={`h-7 px-2.5 text-xs font-semibold transition-colors duration-200 border-y-2 border-x first:border-l-2 last:border-r-2 ${
                                    isActive
                                      ? "bg-mc-green text-white border-mc-green border-t-mc-green-light border-b-mc-green-dark"
                                      : "bg-mc-bg-darker text-mc-text-dim border-mc-border hover:text-mc-text hover:bg-mc-tab-active"
                                  }`}
                                >
                                  {val}
                                </button>
                              );
                            })}
                          </div>
                        </div>

                        {/* Settings toggle */}
                        <button
                          onClick={() => setShowSettings((v) => !v)}
                          className={`ml-auto h-7 px-2.5 text-xs font-semibold flex items-center gap-1.5 transition-colors duration-200 border-2 ${
                            showSettings
                              ? "bg-mc-green text-white border-mc-green border-t-mc-green-light border-b-mc-green-dark"
                              : "bg-mc-bg-darker text-mc-text-dim border-mc-border hover:text-mc-text hover:bg-mc-tab-active"
                          }`}
                        >
                          <Settings2 className="w-3 h-3" />
                          Settings
                        </button>
                      </div>

                      {/* Advanced settings (collapsible) */}
                      <AnimatePresence>
                        {showSettings && (
                          <motion.div
                            initial={{ height: 0, opacity: 0 }}
                            animate={{ height: "auto", opacity: 1 }}
                            exit={{ height: 0, opacity: 0 }}
                            transition={{ duration: 0.2 }}
                            className="overflow-hidden"
                          >
                            <div className="bg-mc-bg-darker border border-mc-border p-3 space-y-3">
                              {/* Sensitivity slider */}
                              <div>
                                <div className="flex items-center justify-between mb-1">
                                  <label className="text-[11px] font-semibold text-mc-text-dim uppercase tracking-wider">
                                    Detection Sensitivity
                                  </label>
                                  <span className="text-xs text-mc-text font-mono">
                                    {(settings.sensitivityThreshold * 1000).toFixed(0)}
                                  </span>
                                </div>
                                <input
                                  type="range"
                                  min={0}
                                  max={30}
                                  step={1}
                                  value={settings.sensitivityThreshold * 1000}
                                  onChange={(e) =>
                                    setSettings((s) => ({
                                      ...s,
                                      sensitivityThreshold: parseInt(e.target.value, 10) / 1000,
                                    }))
                                  }
                                  className="w-full accent-mc-green h-1.5"
                                />
                                <div className="flex justify-between text-[10px] text-mc-text-dim mt-0.5">
                                  <span>Aggressive (0)</span>
                                  <span>Conservative (30)</span>
                                </div>
                              </div>

                              {/* Brightness range */}
                              <div>
                                <div className="flex items-center justify-between mb-1">
                                  <label className="text-[11px] font-semibold text-mc-text-dim uppercase tracking-wider">
                                    Brightness Range
                                  </label>
                                  <span className="text-xs text-mc-text font-mono">
                                    {settings.minBrightness}–{settings.maxBrightness}
                                  </span>
                                </div>
                                <div className="flex gap-3">
                                  <div className="flex-1">
                                    <input
                                      type="range"
                                      min={0}
                                      max={120}
                                      step={5}
                                      value={settings.minBrightness}
                                      onChange={(e) =>
                                        setSettings((s) => ({
                                          ...s,
                                          minBrightness: parseInt(e.target.value, 10),
                                        }))
                                      }
                                      className="w-full accent-mc-green h-1.5"
                                    />
                                    <div className="text-[10px] text-mc-text-dim mt-0.5 text-center">Min</div>
                                  </div>
                                  <div className="flex-1">
                                    <input
                                      type="range"
                                      min={150}
                                      max={255}
                                      step={5}
                                      value={settings.maxBrightness}
                                      onChange={(e) =>
                                        setSettings((s) => ({
                                          ...s,
                                          maxBrightness: parseInt(e.target.value, 10),
                                        }))
                                      }
                                      className="w-full accent-mc-green h-1.5"
                                    />
                                    <div className="text-[10px] text-mc-text-dim mt-0.5 text-center">Max</div>
                                  </div>
                                </div>
                              </div>

                              <p className="text-[10px] text-mc-text-dim leading-relaxed">
                                Detection uses a brightness-invariant green-tint metric.
                                Mossy cobblestone has a green tint, while regular cobblestone
                                is neutral grey. <strong>Sensitivity</strong> controls how much
                                green-excess is required before a cell is confidently classified.
                                <strong> Brightness range</strong> excludes cells that are too
                                dark or too bright (shadows, highlights).
                              </p>
                            </div>
                          </motion.div>
                        )}
                      </AnimatePresence>

                      {/* ── Detection stats ── */}
                      {/* {stats && (
                        <div className="bg-mc-bg-darker border border-mc-border p-3">
                          <div className="flex flex-wrap gap-x-5 gap-y-2 text-xs">
                            <div className="flex items-center gap-1.5">
                              <Image
                                src={TILE_IMAGES[Tile.Mossy]}
                                alt="Mossy"
                                width={14}
                                height={14}
                                className="pixelated"
                              />
                              <span className="text-mc-text">
                                <span className="font-semibold">{stats.mossy}</span>
                                <span className="text-mc-text-dim"> mossy</span>
                              </span>
                            </div>
                            <div className="flex items-center gap-1.5">
                              <Image
                                src={TILE_IMAGES[Tile.Cobble]}
                                alt="Cobble"
                                width={14}
                                height={14}
                                className="pixelated"
                              />
                              <span className="text-mc-text">
                                <span className="font-semibold">{stats.cobble}</span>
                                <span className="text-mc-text-dim"> cobble</span>
                              </span>
                            </div>
                            {stats.unknown > 0 && (
                              <div className="flex items-center gap-1.5">
                                <AlertTriangle className="w-3.5 h-3.5 text-mc-red-text" />
                                <span className="text-mc-red-text">
                                  <span className="font-semibold">{stats.unknown}</span>
                                  <span className="text-mc-text-dim"> unknown</span>
                                </span>
                              </div>
                            )}

                            <div className="ml-auto text-mc-text-dim">
                              Confidence:
                              <span className="text-mc-green-text ml-1 font-semibold">{stats.highConf}</span>
                              <span className="text-mc-text-dim"> / </span>
                              <span className="text-mc-yellow-text font-semibold">{stats.medConf}</span>
                              <span className="text-mc-text-dim"> / </span>
                              <span className="text-mc-red-text font-semibold">{stats.lowConf}</span>
                            </div>
                          </div>
                        </div>
                      )} */}

                      {/* ── Result preview grid ── */}
                      {analysis && (
                        <div>
                          <p className="text-[11px] font-semibold text-mc-text-dim uppercase tracking-wider mb-2">
                            Detected Pattern
                          </p>
                          <div className="inline-grid gap-px bg-mc-bg-darker border border-mc-border p-1"
                            style={{ gridTemplateColumns: `repeat(${cols}, 1fr)` }}
                          >
                            {analysis.map((row, rIdx) =>
                              row.map((cell, cIdx) => (
                                <div
                                  key={`preview-${rIdx}-${cIdx}`}
                                  className="relative w-7 h-7 sm:w-8 sm:h-8"
                                  title={`${TILE_NAMES[cell.tile]} (GE:${(cell.ge * 1000).toFixed(1)} RGB:${cell.avgColor.r.toFixed(0)},${cell.avgColor.g.toFixed(0)},${cell.avgColor.b.toFixed(0)})`}
                                >
                                  <Image
                                    src={TILE_IMAGES[cell.tile]}
                                    alt={TILE_NAMES[cell.tile]}
                                    fill
                                    sizes="32px"
                                    className="pixelated object-cover"
                                  />
                                  {cell.confidence === "low" && (
                                    <div className="absolute inset-0 border-2 border-mc-red/60" />
                                  )}
                                  {cell.confidence === "medium" && (
                                    <div className="absolute inset-0 border border-mc-yellow/40" />
                                  )}
                                </div>
                              ))
                            )}
                          </div>
                        </div>
                      )}
                    </div>
                  </>
                )}
              </div>

              {/* Footer */}
              <div className="flex items-center justify-end gap-2 p-4 border-t border-mc-border">
                <button onClick={handleClose} className="mc-btn mc-btn-outline !py-1.5 !px-4 !text-xs">
                  Cancel
                </button>
                <button
                  onClick={handleApply}
                  disabled={!analysis}
                  className="mc-btn !py-1.5 !px-4 !text-xs"
                >
                  <Check className="w-3.5 h-3.5" />
                  Apply Pattern
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
