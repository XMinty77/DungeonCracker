"use client";

import { motion } from "framer-motion";
import { Crosshair, Settings } from "lucide-react";
import {
  MC_VERSIONS,
  BIOMES,
  BIOME_LABELS,
  type MCVersion,
  type Biome,
} from "@/lib/types";

interface OptionsFormProps {
  spawnerX: string;
  spawnerY: string;
  spawnerZ: string;
  version: MCVersion;
  biome: Biome;
  onSpawnerXChange: (v: string) => void;
  onSpawnerYChange: (v: string) => void;
  onSpawnerZChange: (v: string) => void;
  onVersionChange: (v: MCVersion) => void;
  onBiomeChange: (v: Biome) => void;
}

export function OptionsForm({
  spawnerX,
  spawnerY,
  spawnerZ,
  version,
  biome,
  onSpawnerXChange,
  onSpawnerYChange,
  onSpawnerZChange,
  onVersionChange,
  onBiomeChange,
}: OptionsFormProps) {
  return (
    <motion.div
      initial={{ opacity: 0, x: 30 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5, delay: 0.15, ease: "easeOut" }}
      className="space-y-4"
    >
      {/* ── Spawner Coordinates ── */}
      <div>
        <div className="flex items-center gap-2 mb-3">
          <Crosshair className="w-4 h-4 text-mc-green-text" />
          <h2 className="font-minecraft text-sm text-mc-text-highlight tracking-wide">
            Spawner Coordinates
          </h2>
        </div>

        <div className="mc-panel p-4">
          <div className="grid grid-cols-3 gap-3">
            {[
              {
                label: "X",
                value: spawnerX,
                onChange: onSpawnerXChange,
                placeholder: "320",
              },
              {
                label: "Y",
                value: spawnerY,
                onChange: onSpawnerYChange,
                placeholder: "29",
              },
              {
                label: "Z",
                value: spawnerZ,
                onChange: onSpawnerZChange,
                placeholder: "-418",
              },
            ].map(({ label, value, onChange, placeholder }) => (
              <div key={label}>
                <label className="block text-[11px] font-semibold text-mc-text-dim mb-1 uppercase tracking-wider">
                  {label}
                </label>
                <input
                  type="number"
                  value={value}
                  onChange={(e) => onChange(e.target.value)}
                  placeholder={placeholder}
                  className="mc-input"
                />
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* ── Generation Options ── */}
      <div>
        <div className="flex items-center gap-2 mb-3">
          <Settings className="w-4 h-4 text-mc-green-text" />
          <h2 className="font-minecraft text-sm text-mc-text-highlight tracking-wide">
            Options
          </h2>
        </div>

        <div className="mc-panel p-4 space-y-3">
          <div>
            <label className="block text-[11px] font-semibold text-mc-text-dim mb-1 uppercase tracking-wider">
              Minecraft Version
            </label>
            <select
              value={version}
              onChange={(e) =>
                onVersionChange(e.target.value as MCVersion)
              }
              className="mc-select"
            >
              {MC_VERSIONS.map((v) => (
                <option key={v} value={v}>
                  {v}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-[11px] font-semibold text-mc-text-dim mb-1 uppercase tracking-wider">
              Biome
            </label>
            <select
              value={biome}
              onChange={(e) => onBiomeChange(e.target.value as Biome)}
              className="mc-select"
            >
              {BIOMES.map((b) => (
                <option key={b} value={b}>
                  {BIOME_LABELS[b]}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
