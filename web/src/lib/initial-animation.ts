/**
 * Module-level flag to track whether the initial page-load animation has played.
 * Once set to `true`, all entry animations are suppressed on subsequent mounts
 * (e.g. tab switches).
 *
 * Usage:
 *   import { hasAnimated, markAnimated } from "@/lib/initial-animation";
 *   <motion.div initial={hasAnimated() ? false : { opacity: 0 }} ... />
 *   // After initial animation completes:
 *   useEffect(() => { const t = setTimeout(markAnimated, 800); return () => clearTimeout(t); }, []);
 */

let _done = false;

/** Returns `true` if the initial page animation has already played. */
export function hasAnimated(): boolean {
  return _done;
}

/** Mark the initial animation as complete. All future mounts will skip entry animation. */
export function markAnimated(): void {
  _done = true;
}
