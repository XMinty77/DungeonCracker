"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import {
  Copy,
  Check,
  Download,
  Trophy,
  Key,
  Globe,
  Lock,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import type { CrackResult } from "@/lib/types";

interface ResultsPanelProps {
  result: CrackResult;
}

interface TabDef {
  id: string;
  title: string;
  seeds: string[];
  icon: LucideIcon;
  accentColor: string;
}

export function ResultsPanel({ result }: ResultsPanelProps) {
  const tabs: TabDef[] = [
    {
      id: "dungeon",
      title: "Dungeon Seeds",
      seeds: result.dungeon_seeds,
      icon: Key,
      accentColor: "#4A9FD9",
    },
    {
      id: "structure",
      title: "Structure Seeds",
      seeds: result.structure_seeds,
      icon: Lock,
      accentColor: "#FFC42B",
    },
    {
      id: "world",
      title: "World Seeds",
      seeds: result.world_seeds,
      icon: Globe,
      accentColor: "#6CC349",
    },
  ];

  // Default to first tab that has seeds, else first tab
  const initialTab =
    tabs.find((t) => t.seeds.length > 0)?.id ?? tabs[0].id;
  const [activeTab, setActiveTab] = useState(initialTab);
  const [copied, setCopied] = useState(false);

  const active = tabs.find((t) => t.id === activeTab)!;

  const copySeeds = () => {
    if (active.seeds.length === 0) return;
    navigator.clipboard.writeText(active.seeds.join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const saveResults = () => {
    const lines = [
      "=== Dungeon Cracker Results ===",
      `Date: ${new Date().toISOString()}`,
      "",
      `Dungeon Seeds (${result.dungeon_seeds.length}):`,
      ...result.dungeon_seeds,
      "",
      `Structure Seeds (${result.structure_seeds.length}):`,
      ...result.structure_seeds,
      "",
      `World Seeds (${result.world_seeds.length}):`,
      ...result.world_seeds,
    ];
    const blob = new Blob([lines.join("\n")], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `dungeon-crack-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const totalSeeds =
    result.dungeon_seeds.length +
    result.structure_seeds.length +
    result.world_seeds.length;

  return (
    <motion.div
      initial={{ opacity: 0, x: 30 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5, delay: 0.1, ease: "easeOut" }}
    >
      {/* Section heading */}
      <div className="flex items-center gap-2 mb-3">
        <Trophy className="w-4 h-4 text-mc-green-text" />
        <h2 className="font-minecraft text-sm text-mc-text-highlight tracking-wide">
          Results
        </h2>
      </div>

      <div
        className="mc-panel overflow-hidden"
        style={{ borderColor: "#3C8527" }}
      >
        {/* Green accent top strip */}
        <div className="h-0.5 bg-mc-green-light" />

        {/* Tab bar */}
        <div className="flex items-stretch border-b border-mc-border">
          {/* Tabs */}
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
                    borderBottomColor: isActive ? tab.accentColor : undefined,
                  }}
                >
                  <Icon
                    className="w-3.5 h-3.5 flex-shrink-0"
                    style={{ color: isActive ? tab.accentColor : undefined }}
                  />
                  <span className="hidden sm:inline">{tab.title}</span>
                  <span
                    className="text-[10px] font-bold px-1.5 py-px min-w-[22px] text-center border"
                    style={{
                      backgroundColor: tab.accentColor + "22",
                      color: tab.accentColor,
                      borderColor: tab.accentColor + "44",
                    }}
                  >
                    {tab.seeds.length}
                  </span>
                </button>
              );
            })}
          </div>

          {/* Action icons */}
          <div className="flex items-center gap-0.5 px-2 bg-mc-bg">
            {active.seeds.length > 0 && (
              <button
                onClick={copySeeds}
                className="p-1.5 hover:bg-mc-tab-active transition-colors duration-200 cursor-pointer"
                title="Copy seeds to clipboard"
              >
                {copied ? (
                  <Check className="w-3.5 h-3.5 text-mc-green-text" />
                ) : (
                  <Copy className="w-3.5 h-3.5 text-mc-text-dim hover:text-mc-text" />
                )}
              </button>
            )}
            {totalSeeds > 0 && (
              <button
                onClick={saveResults}
                className="p-1.5 hover:bg-mc-tab-active transition-colors duration-200 cursor-pointer"
                title="Download all results as .txt"
              >
                <Download className="w-3.5 h-3.5 text-mc-text-dim hover:text-mc-text" />
              </button>
            )}
          </div>
        </div>

        {/* Tab content */}
        <div className="bg-mc-bg-darker max-h-56 overflow-y-auto">
          {active.seeds.length === 0 ? (
            <p className="text-[11px] text-mc-text-dim italic py-4 px-3 text-center">
              No seeds found
            </p>
          ) : (
            <div className="py-1">
              {active.seeds.map((seed, i) => (
                <motion.p
                  key={`${activeTab}-${seed}-${i}`}
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
    </motion.div>
  );
}
