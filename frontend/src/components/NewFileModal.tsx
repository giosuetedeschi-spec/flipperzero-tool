import { useState } from "react";

interface Props {
  currentPath: string;
  onClose: () => void;
  onCreate: (name: string, ext: string) => Promise<void>;
}

const EXTS = [
  { value: "sub", label: ".sub (Sub-GHz)" },
  { value: "ir", label: ".ir (Infrared)" },
  { value: "nfc", label: ".nfc (NFC)" },
  { value: "txt", label: ".txt (BadUSB)" },
];

export default function NewFileModal({ onClose, onCreate }: Props) {
  const [name, setName] = useState("");
  const [ext, setExt] = useState("sub");
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    if (!name.trim()) return;
    setBusy(true);
    try {
      await onCreate(name.trim(), ext);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center gap-3">
      <span className="text-sm text-gray-400">New file:</span>
      <input
        type="text"
        placeholder="filename"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && submit()}
        className="px-3 py-1 bg-gray-700 rounded text-sm w-48 placeholder-gray-500"
        autoFocus
        disabled={busy}
      />
      <select
        value={ext}
        onChange={(e) => setExt(e.target.value)}
        className="px-2 py-1 bg-gray-700 rounded text-sm"
        disabled={busy}
      >
        {EXTS.map((e) => (
          <option key={e.value} value={e.value}>{e.label}</option>
        ))}
      </select>
      <button
        onClick={submit}
        disabled={busy || !name.trim()}
        className="px-4 py-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 rounded text-sm font-medium"
      >
        {busy ? "Creating..." : "Create"}
      </button>
      <button
        onClick={onClose}
        className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm"
        disabled={busy}
      >
        Cancel
      </button>
    </div>
  );
}
