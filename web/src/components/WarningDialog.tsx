"use client";

import { motion, AnimatePresence } from "framer-motion";
import { AlertTriangle, X } from "lucide-react";
import { useEffect } from "react";

export interface WarningDialogAction {
  label: string;
  /** Additional CSS class, e.g. "mc-btn-red" */
  className?: string;
  onClick: () => void;
}

interface WarningDialogProps {
  open: boolean;
  title: string;
  /** Lines of body text */
  body: string[];
  /** Optional detail text shown in a dimmer, smaller style */
  detail?: string;
  actions: WarningDialogAction[];
  onClose: () => void;
}

export function WarningDialog({
  open,
  title,
  body,
  detail,
  actions,
  onClose,
}: WarningDialogProps) {
  // Close on Escape
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center p-4"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
        >
          {/* Backdrop */}
          <div
            className="absolute inset-0 bg-black/60"
            onClick={onClose}
          />

          {/* Dialog */}
          <motion.div
            className="relative mc-panel p-5 max-w-md w-full space-y-4 shadow-2xl"
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.95, opacity: 0 }}
            transition={{ duration: 0.15 }}
          >
            {/* Close button */}
            <button
              onClick={onClose}
              className="absolute top-3 right-3 p-1 text-mc-text-dim hover:text-mc-text transition-colors cursor-pointer"
            >
              <X className="w-4 h-4" />
            </button>

            {/* Title */}
            <div className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-mc-yellow flex-shrink-0" />
              <h3 className="font-minecraft text-sm text-mc-text-highlight tracking-wide">
                {title}
              </h3>
            </div>

            {/* Body */}
            <div className="space-y-2">
              {body.map((line, i) => (
                <p key={i} className="text-xs text-mc-text leading-relaxed">
                  {line}
                </p>
              ))}
              {detail && (
                <p className="text-[11px] text-mc-text-dim leading-relaxed mt-1">
                  {detail}
                </p>
              )}
            </div>

            {/* Actions */}
            <div className="flex items-center justify-end gap-2 pt-1">
              {actions.map((action, i) => (
                <button
                  key={i}
                  onClick={action.onClick}
                  className={`mc-btn !py-2 !px-4 !text-xs ${action.className ?? ""}`}
                >
                  {action.label}
                </button>
              ))}
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
