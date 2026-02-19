"use client";

import { motion } from "framer-motion";
import Image from "next/image";
import { Github } from "lucide-react";

export function Header() {
  return (
    <motion.header
      initial={{ y: -50, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ duration: 0.5, ease: "easeOut" }}
      className="sticky top-0 z-50 bg-mc-titlebar border-b-2 border-mc-border"
    >
      <div className="max-w-7xl mx-auto flex items-center gap-3.5 px-5 py-3">
        {/* Icon */}
        <motion.div
          whileHover={{ rotate: 12, scale: 1.1 }}
          transition={{ type: "spring", stiffness: 400, damping: 15 }}
        >
          <Image
            src="/icon.png"
            alt="Dungeon Cracker"
            width={32}
            height={32}
            className="pixelated"
            priority
          />
        </motion.div>

        {/* Title */}
        <div className="flex flex-col">
          <h1 className="font-minecraft text-lg leading-tight text-mc-text-highlight tracking-wide">
            Dungeon Cracker
          </h1>
          <p className="text-[11px] text-mc-text-dim leading-tight">
            In-browser seed cracker utility
          </p>
        </div>

        {/* Right-side links */}
        <div className="ml-auto flex items-center gap-2">
          <a
            href="https://github.com/XMinty77/DungeonCracker"
            target="_blank"
            rel="noopener noreferrer"
            className="mc-btn mc-btn-outline !py-1.5 !px-3 !text-xs !gap-1.5"
          >
            <Github className="w-3.5 h-3.5" />
            GitHub
          </a>
        </div>
      </div>
    </motion.header>
  );
}
