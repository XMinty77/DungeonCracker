import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  basePath: "/DungeonCracker",
  images: {
    unoptimized: true,
  },
};

export default nextConfig;
