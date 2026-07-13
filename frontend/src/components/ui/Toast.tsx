import { useEffect, useState } from "react";
import { subscribeToasts, type ToastMsg } from "../../lib/toastStore";

export function ToastContainer() {
  const [toasts, setToasts] = useState<ToastMsg[]>([]);

  useEffect(() => {
    return subscribeToasts((msg) => {
      setToasts((prev) => [...prev, msg]);
      setTimeout(() => setToasts((prev) => prev.filter((t) => t.id !== msg.id)), 4000);
    });
  }, []);

  const styles = {
    success: "bg-emerald-900/95 border-emerald-600 text-emerald-100",
    error: "bg-red-950/95 border-red-700 text-red-100",
    info: "bg-slate-950/95 border-slate-600 text-slate-100",
  };

  const icons = {
    success: "✓",
    error: "⚠️",
    info: "ℹ️",
  };

  return (
    <div className="fixed bottom-4 right-4 flex flex-col gap-3 z-50 pointer-events-none">
      {toasts.map((t) => (
        <div key={t.id} className={`min-w-[260px] max-w-sm rounded-2xl border px-4 py-3 shadow-2xl backdrop-blur-sm pointer-events-auto ${styles[t.type]}`} role="status" aria-live="polite">
          <div className="flex gap-3 items-start">
            <span className="text-lg">{icons[t.type]}</span>
            <div className="text-sm leading-5 break-words whitespace-pre-wrap">{t.text}</div>
          </div>
        </div>
      ))}
    </div>
  );
}
