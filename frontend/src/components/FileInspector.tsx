import SubGhzViewerAdvanced from "./SubGhzViewerAdvanced";
import NfcAnalyzerAdvanced from "./NfcAnalyzerAdvanced";
import IrDatabase from "./IrDatabase";
import FilePreview from "./FilePreview";
import type { FileInfo } from "../services/tauri";

/**
 * Picks the right viewer for a Flipper file.
 *
 * These viewers already existed but nothing rendered them, so a `.sub` file
 * opened as plain text with its waveform, protocol database and Pronto export
 * sitting unreachable in the bundle. Routing by extension is all that was
 * missing.
 */

interface Props {
  file: FileInfo | null;
  content: string;
  onClose: () => void;
}

/** Lowercase extension without the dot, or "" when there is none. */
function extensionOf(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot === -1 ? "" : name.slice(dot + 1).toLowerCase();
}

export default function FileInspector({ file, content, onClose }: Props) {
  if (!file) return null;

  switch (extensionOf(file.name)) {
    case "sub":
      return <SubGhzViewerAdvanced content={content} fileName={file.name} />;
    case "nfc":
      return <NfcAnalyzerAdvanced content={content} fileName={file.name} />;
    case "ir":
      return <IrDatabase content={content} fileName={file.name} />;
    default:
      // Everything else -- .rfid, .ibtn, .txt, .fap -- falls back to the generic
      // preview rather than showing nothing.
      return <FilePreview file={file} content={content} onClose={onClose} />;
  }
}
