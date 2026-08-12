import { describe, expect, it as test, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import FileInspector from "./FileInspector";
import type { FileInfo } from "../services/tauri";

/**
 * The viewers this routes to are heavy -- CodeMirror, protocol databases,
 * waveform rendering -- and none of that is what is under test. Stubbing them
 * keeps these tests about the one thing FileInspector decides: which viewer a
 * file gets.
 */
vi.mock("./SubGhzViewerAdvanced", () => ({
  default: ({ fileName }: { fileName: string }) => <div data-testid="viewer-subghz">{fileName}</div>,
}));
vi.mock("./NfcAnalyzerAdvanced", () => ({
  default: ({ fileName }: { fileName: string }) => <div data-testid="viewer-nfc">{fileName}</div>,
}));
vi.mock("./IrDatabase", () => ({
  default: ({ fileName }: { fileName: string }) => <div data-testid="viewer-ir">{fileName}</div>,
}));
vi.mock("./FilePreview", () => ({
  default: ({ file }: { file: FileInfo | null }) => (
    <div data-testid="viewer-fallback">{file?.name}</div>
  ),
}));

function file(name: string): FileInfo {
  return { path: `/ext/${name}`, name, size: 10, is_dir: false, modified: null };
}

describe("FileInspector", () => {
  test("routes each Flipper format to the viewer built for it", () => {
    // These viewers existed and rendered nowhere; extension routing is the whole
    // reason they are reachable at all.
    const cases: [string, string][] = [
      ["gate.sub", "viewer-subghz"],
      ["badge.nfc", "viewer-nfc"],
      ["tv.ir", "viewer-ir"],
    ];

    for (const [name, testId] of cases) {
      const { unmount } = render(<FileInspector file={file(name)} content="" onClose={() => {}} />);
      expect(screen.getByTestId(testId), name).toBeInTheDocument();
      unmount();
    }
  });

  test("anything else falls back to the generic preview rather than nothing", () => {
    for (const name of ["door.rfid", "key.ibtn", "notes.txt", "app.fap"]) {
      const { unmount } = render(<FileInspector file={file(name)} content="" onClose={() => {}} />);
      expect(screen.getByTestId("viewer-fallback"), name).toBeInTheDocument();
      unmount();
    }
  });

  test("matches the extension regardless of case", () => {
    render(<FileInspector file={file("GATE.SUB")} content="" onClose={() => {}} />);
    expect(screen.getByTestId("viewer-subghz")).toBeInTheDocument();
  });

  test("a file with no extension still gets the preview", () => {
    render(<FileInspector file={file("README")} content="" onClose={() => {}} />);
    expect(screen.getByTestId("viewer-fallback")).toBeInTheDocument();
  });

  test("a name that only looks like an extension is matched on the last dot", () => {
    render(<FileInspector file={file("my.sub.backup.nfc")} content="" onClose={() => {}} />);
    expect(screen.getByTestId("viewer-nfc")).toBeInTheDocument();
  });

  test("renders nothing when no file is selected", () => {
    const { container } = render(<FileInspector file={null} content="" onClose={() => {}} />);
    expect(container).toBeEmptyDOMElement();
  });
});
