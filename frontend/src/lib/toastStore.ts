export interface ToastMsg {
  id: number;
  text: string;
  type: "success" | "error" | "info";
}

let toastId = 0;
let listener: ((msg: ToastMsg) => void) | null = null;

export function showToast(text: string, type: "info" | "success" | "error" = "info") {
  listener?.({ id: ++toastId, text, type });
}

export function subscribeToasts(fn: (msg: ToastMsg) => void): () => void {
  listener = fn;
  return () => { listener = null; };
}
