import { useEffect, useState } from "react";

export interface ToastMsg {
  id: number;
  text: string;
  type: "success" | "error" | "info";
}

let toastId = 0;
let addToastFn: ((msg: Omit<ToastMsg, "id">) => void) | null = null;

export function showToast(text: string, type: "info" | "success" | "error" = "info") {
  addToastFn?.({ text, type });
}

export function ToastContainer() {
  const [toasts, setToasts] = useState<ToastMsg[]>([]);

  useEffect(() => {
    addToastFn = (msg) => {
      const id = ++toastId;
      setToasts((prev) => [...prev, { ...msg, id }]);
      setTimeout(() => setToasts((prev) => prev.filter((t) => t.id !== id)), 3000);
    };
    return () => { addToastFn = null; };
  }, []);

  const colors = {
    success: "bg-emerald-800 border-emerald-600 text-emerald-100",
    error: "bg-red-900 border-red-700 text-red-100",
    info: "bg-blue-900 border-blue-700 text-blue-100",
  };

  return (
    <div className="fixed bottom-4 right-4 flex flex-col gap-2 z-50 pointer-events-none">
      {toasts.map((t) => (
        <div key={t.id} className={`px-4 py-2 rounded-lg border text-sm shadow-lg pointer-events-auto ${colors[t.type]}`}>
          {t.text}
        </div>
      ))}
    </div>
  );
}
