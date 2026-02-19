"use client";

import { motion } from "framer-motion";

interface Particle {
  id: number;
  x: number;
  size: number;
  dur: number;
  delay: number;
  color: string;
}

const COLORS = ["#3C8527", "#52A535", "#2A641C", "#6CC349", "#FFC42B"];

const PARTICLES: Particle[] = Array.from({ length: 30 }, (_, i) => ({
  id: i,
  x: ((i * 37 + 13) % 100),
  size: 2 + ((i * 7 + 3) % 30) / 10,
  dur: 12 + ((i * 11 + 5) % 18),
  delay: ((i * 13 + 7) % 15),
  color: COLORS[i % COLORS.length],
}));

export function ParticleBackground() {
  return (
    <div className="fixed inset-0 overflow-hidden pointer-events-none z-0">
      {PARTICLES.map((p) => (
        <motion.div
          key={p.id}
          className="absolute"
          style={{
            left: `${p.x}%`,
            bottom: -8,
            width: p.size,
            height: p.size,
            backgroundColor: p.color,
          }}
          animate={{
            y: [0, -1200],
            opacity: [0, 0.5, 0.4, 0],
            rotate: [0, 180],
          }}
          transition={{
            duration: p.dur,
            delay: p.delay,
            repeat: Infinity,
            ease: "linear",
          }}
        />
      ))}
    </div>
  );
}
