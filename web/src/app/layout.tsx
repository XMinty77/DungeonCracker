import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Dungeon Cracker",
  description:
    "Minecraft dungeon floor seed cracker — crack dungeon seeds from floor patterns.",
  icons: {
    icon: "/DungeonCracker/icon.png",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased">{children}</body>
    </html>
  );
}
