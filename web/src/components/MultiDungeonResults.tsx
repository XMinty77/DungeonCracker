"use client";

import { useState, useMemo } from "react";
import { motion } from "framer-motion";
import {
  Copy,
  Check,
  Download,
  Trophy,
  Key,
  Globe,
  Lock,
  Layers,
} from "lucide-react";
import type { CrackResult } from "@/lib/types";
import { hasAnimated } from "@/lib/initial-animation";

/* ── Single-dungeon results (collapsed inline view) ── */

interface DungeonResultCardProps {
  index: number;
  label: string;
  result: CrackResult;
}

function DungeonResultCard({ index, label, result }: DungeonResultCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [activeTab, setActiveTab] = useState<"dungeon" | "structure" | "world">("world");
  const [copied, setCopied] = useState(false);

  const tabs = [
    { id: "dungeon" as const, title: "Dungeon", seeds: result.dungeon_seeds, icon: Key, color: "#4A9FD9" },
    { id: "structure" as const, title: "Structure", seeds: result.structure_seeds, icon: Lock, color: "#FFC42B" },
    { id: "world" as const, title: "World", seeds: result.world_seeds, icon: Globe, color: "#6CC349" },
  ];

  const active = tabs.find((t) => t.id === activeTab)!;

  const copySeeds = () => {
    if (active.seeds.length === 0) return;
    navigator.clipboard.writeText(active.seeds.join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="mc-panel overflow-hidden" style={{ borderColor: "#3C6B8A" }}>
      <button
        onClick={() => setExpanded((p) => !p)}
        className="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-mc-tab-active transition-colors duration-200 cursor-pointer"
      >
        <span className="text-[10px] font-bold px-1.5 py-px bg-mc-bg-darker border border-mc-border text-mc-text-dim">
          #{index + 1}
        </span>
        <span className="text-xs font-semibold text-mc-text-highlight flex-1 min-w-0 truncate">
          {label}
        </span>
        <div className="flex items-center gap-2">
          {tabs.map((tab) => (
            <span
              key={tab.id}
              className="text-[10px] font-bold px-1.5 py-px border"
              style={{
                backgroundColor: tab.color + "22",
                color: tab.color,
                borderColor: tab.color + "44",
              }}
            >
              {tab.seeds.length}
            </span>
          ))}
        </div>
        <span className="text-mc-text-dim text-xs">{expanded ? "▲" : "▼"}</span>
      </button>

      {expanded && (
        <div className="border-t border-mc-border">
          {/* Tab bar */}
          <div className="flex items-stretch border-b border-mc-border">
            <div className="flex flex-1 min-w-0">
              {tabs.map((tab) => {
                const Icon = tab.icon;
                const isActive = tab.id === activeTab;
                return (
                  <button
                    key={tab.id}
                    onClick={() => {
                      setActiveTab(tab.id);
                      setCopied(false);
                    }}
                    className={`flex items-center gap-1.5 px-3 py-2 text-xs font-semibold transition-colors duration-200 cursor-pointer border-b-2 ${
                      isActive
                        ? "bg-mc-tab-active text-mc-text-highlight"
                        : "bg-mc-bg text-mc-text-dim hover:bg-mc-tab-active hover:text-mc-text border-transparent"
                    }`}
                    style={{
                      borderBottomColor: isActive ? tab.color : undefined,
                    }}
                  >
                    <Icon
                      className="w-3.5 h-3.5 flex-shrink-0"
                      style={{ color: isActive ? tab.color : undefined }}
                    />
                    <span className="hidden sm:inline">{tab.title}</span>
                    <span
                      className="text-[10px] font-bold px-1.5 py-px min-w-[22px] text-center border"
                      style={{
                        backgroundColor: tab.color + "22",
                        color: tab.color,
                        borderColor: tab.color + "44",
                      }}
                    >
                      {tab.seeds.length}
                    </span>
                  </button>
                );
              })}
            </div>
            {active.seeds.length > 0 && (
              <div className="flex items-center gap-0.5 px-2 bg-mc-bg">
                <button
                  onClick={copySeeds}
                  className="p-1.5 hover:bg-mc-tab-active transition-colors duration-200 cursor-pointer"
                  title="Copy seeds"
                >
                  {copied ? (
                    <Check className="w-3.5 h-3.5 text-mc-green-text" />
                  ) : (
                    <Copy className="w-3.5 h-3.5 text-mc-text-dim hover:text-mc-text" />
                  )}
                </button>
              </div>
            )}
          </div>

          {/* Seed list */}
          <div className="bg-mc-bg-darker max-h-40 overflow-y-auto">
            {active.seeds.length === 0 ? (
              <p className="text-[11px] text-mc-text-dim italic py-3 px-3 text-center">
                No seeds found
              </p>
            ) : (
              <div className="py-1">
                {active.seeds.map((seed, i) => (
                  <p
                    key={`${activeTab}-${seed}-${i}`}
                    className="text-xs font-mono text-mc-text py-0.5 px-3 hover:bg-mc-tab-active transition-colors duration-200 cursor-default select-all"
                  >
                    {seed}
                  </p>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

/* ── Combined multi-dungeon results panel ── */

interface MultiDungeonResultsProps {
  results: { label: string; result: CrackResult }[];
}

export function MultiDungeonResults({ results }: MultiDungeonResultsProps) {
  const [copied, setCopied] = useState(false);

  // Compute the intersection of world seeds across all dungeons
  const commonWorldSeeds = useMemo(() => {
    if (results.length === 0) return [];
    if (results.length === 1) return results[0].result.world_seeds;

    // Start with a set from the first result
    let intersection = new Set(results[0].result.world_seeds);
    for (let i = 1; i < results.length; i++) {
      const next = new Set(results[i].result.world_seeds);
      intersection = new Set([...intersection].filter((s) => next.has(s)));
    }
    return [...intersection];
  }, [results]);

  const copyCommon = () => {
    if (commonWorldSeeds.length === 0) return;
    navigator.clipboard.writeText(commonWorldSeeds.join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const saveAllResults = () => {
    const lines: string[] = [
      "=== Dungeon Cracker — Multi-Dungeon Results ===",
      `Date: ${new Date().toISOString()}`,
      `Dungeons cracked: ${results.length}`,
      "",
    ];

    results.forEach(({ label, result }, i) => {
      lines.push(`── Dungeon #${i + 1}: ${label} ──`);
      lines.push(`  Dungeon Seeds (${result.dungeon_seeds.length}):`);
      result.dungeon_seeds.forEach((s) => lines.push(`    ${s}`));
      lines.push(`  Structure Seeds (${result.structure_seeds.length}):`);
      result.structure_seeds.forEach((s) => lines.push(`    ${s}`));
      lines.push(`  World Seeds (${result.world_seeds.length}):`);
      result.world_seeds.forEach((s) => lines.push(`    ${s}`));
      lines.push("");
    });

    lines.push(`── Common World Seeds (${commonWorldSeeds.length}) ──`);
    commonWorldSeeds.forEach((s) => lines.push(`  ${s}`));

    const blob = new Blob([lines.join("\n")], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `dungeon-crack-multi-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <motion.div
      initial={hasAnimated() ? false : { opacity: 0, x: 30 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5, delay: 0.1, ease: "easeOut" }}
      className="space-y-4"
    >
      {/* ── Common World Seeds (the main payoff) ── */}
      <div>
        <div className="flex items-center gap-2 mb-3">
          <Layers className="w-4 h-4 text-mc-green-text" />
          <h2 className="font-minecraft text-sm text-mc-text-highlight tracking-wide">
            Common World Seeds
          </h2>
          <span className="text-[10px] font-bold px-1.5 py-px border bg-[#6CC34922] text-[#6CC349] border-[#6CC34944]">
            {commonWorldSeeds.length}
          </span>
        </div>

        <div className="mc-panel overflow-hidden" style={{ borderColor: "#3C8527" }}>
          <div className="h-0.5 bg-mc-green-light" />

          {/* Actions bar */}
          <div className="flex items-center justify-between px-3 py-2 border-b border-mc-border bg-mc-bg">
            <span className="text-xs text-mc-text-dim">
              {results.length === 1
                ? "Crack more dungeons to find common world seeds"
                : `Intersection of ${results.length} dungeons`}
            </span>
            <div className="flex items-center gap-0.5">
              {commonWorldSeeds.length > 0 && (
                <button
                  onClick={copyCommon}
                  className="p-1.5 hover:bg-mc-tab-active transition-colors duration-200 cursor-pointer"
                  title="Copy common world seeds"
                >
                  {copied ? (
                    <Check className="w-3.5 h-3.5 text-mc-green-text" />
                  ) : (
                    <Copy className="w-3.5 h-3.5 text-mc-text-dim hover:text-mc-text" />
                  )}
                </button>
              )}
              <button
                onClick={saveAllResults}
                className="p-1.5 hover:bg-mc-tab-active transition-colors duration-200 cursor-pointer"
                title="Download all results as .txt"
              >
                <Download className="w-3.5 h-3.5 text-mc-text-dim hover:text-mc-text" />
              </button>
            </div>
          </div>

          {/* Seed list */}
          <div className="bg-mc-bg-darker max-h-56 overflow-y-auto">
            {commonWorldSeeds.length === 0 ? (
              <p className="text-[11px] text-mc-text-dim italic py-4 px-3 text-center">
                {results.length < 2
                  ? "Add and crack multiple dungeons to narrow down world seeds"
                  : "No world seeds in common across all dungeons"}
              </p>
            ) : (
              <div className="py-1">
                {commonWorldSeeds.map((seed, i) => (
                  <motion.p
                    key={`common-${seed}-${i}`}
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{
                      delay: Math.min(i * 0.01, 0.3),
                      duration: 0.15,
                    }}
                    className="text-xs font-mono text-mc-text py-0.5 px-3 hover:bg-mc-tab-active transition-colors duration-200 cursor-default select-all"
                  >
                    {seed}
                  </motion.p>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* ── Per-dungeon results ── */}
      <div>
        <div className="flex items-center gap-2 mb-3">
          <Trophy className="w-4 h-4 text-mc-green-text" />
          <h2 className="font-minecraft text-sm text-mc-text-highlight tracking-wide">
            Per-Dungeon Results
          </h2>
        </div>

        <div className="space-y-2">
          {results.map(({ label, result }, i) => (
            <DungeonResultCard key={i} index={i} label={label} result={result} />
          ))}
        </div>
      </div>
    </motion.div>
  );
}
