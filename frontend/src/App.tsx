import { useState } from "react";
import { serialDisconnect, createFileFromTemplate, moveFile, type FileInfo } from "./services/tauri";
import { useDirectory } from "./hooks/useDirectory";
import { useEditor } from "./hooks/useEditor";
import { useDragDrop } from "./hooks/useDragDrop";
import NewFileModal from "./components/NewFileModal";
import DevicePanel from "./components/DevicePanel";
import { ToastContainer } from "./components/ui/Toast";
import { showToast } from "./lib/toastStore";
import { getErrorMessage, signalsPerformAction } from "./services/tauri";
import type { ActionId, Signal } from "./types/signals";
import AppShell from "./shell/AppShell";
import type { Section } from "./shell/sections";
import { useIsMobileLayout } from "./shell/useBreakpoint";
import RadarView from "./views/RadarView";
import FilesView from "./views/FilesView";
import ToolsView from "./views/ToolsView";
import DeviceView from "./views/DeviceView";
import SettingsView from "./views/SettingsView";
import { useT } from "./i18n/useT";

type ViewMode = "local" | "serial";

export default function App() {
  const { t } = useT();
  const isMobile = useIsMobileLayout();

  // A phone opens on the Radar -- it is what the mobile app is for. Desktop
  // opens on the file browser, which is what it has always done.
  const [section, setSection] = useState<Section>(() =>
    typeof window !== "undefined" && window.innerWidth < 768 ? "radar" : "files",
  );

  const [viewMode, setViewMode] = useState<ViewMode>("local");
  const [serialConnected, setSerialConnected] = useState(false);
  const [mockMode, setMockMode] = useState(false);
  const [showNewFile, setShowNewFile] = useState(false);

  const effectiveSerialConnected = serialConnected || mockMode;

  const dir = useDirectory(viewMode, effectiveSerialConnected, mockMode);
  const editor = useEditor(viewMode, mockMode);
  const dnd = useDragDrop(async (file: FileInfo, targetPath: string) => {
    try {
      await moveFile(file.path, targetPath + "/" + file.name);
      dir.refresh();
      showToast(`Moved ${file.name} → ${targetPath}`, "success");
    } catch (err) {
      showToast(err instanceof Error ? err.message : String(err), "error");
    }
  });

  const handleDisconnect = async () => {
    try {
      await serialDisconnect();
    } catch {
      /* already gone; nothing to report */
    }
    setSerialConnected(false);
    dir.setCurrentPath(localStorage.getItem("flipper_root_path") ?? "");
    editor.closeAll();
    showToast(t("device.disconnected"), "info");
  };

  const handleSwitchMode = (mode: ViewMode) => {
    if (mode !== viewMode && serialConnected) void handleDisconnect();
    setViewMode(mode);
    if (mode !== "serial") setMockMode(false);
    editor.closeAll();
    dir.setError(null);
  };

  const handleNavigate = (info: FileInfo) => {
    if (!info.is_dir) return;
    dir.setCurrentPath(info.path);
    dir.setSearchQuery("");
    editor.closeAll();
  };

  // Actions on a detected signal. Every outcome is reported: an action that
  // cannot run yet says why, rather than looking like a button that does
  // nothing.
  const handleSignalAction = async (signal: Signal, action: ActionId) => {
    try {
      const outcome = await signalsPerformAction(signal.id, action);
      switch (outcome.kind) {
        case "saved":
          showToast(`${t("action.save")}: ${outcome.path}`, "success");
          break;
        case "exported":
          await navigator.clipboard?.writeText(outcome.contents).catch(() => {});
          showToast(`${t("action.export")}: ${outcome.filename}`, "success");
          break;
        case "analysis":
          showToast(`${t("action.analyze")}: entropy ${outcome.entropy.toFixed(2)}`, "info");
          break;
        case "unavailable":
          showToast(outcome.detail, "info");
          break;
      }
    } catch (err) {
      showToast(getErrorMessage(err), "error");
    }
  };

  const handleCreateFile = async (name: string, ext: string) => {
    try {
      await createFileFromTemplate(dir.currentPath + "/" + name, ext);
      setShowNewFile(false);
      dir.refresh();
      showToast(`Created ${name}.${ext}`, "success");
    } catch (err) {
      dir.setError(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <AppShell section={section} onNavigate={setSection}>
      {/* The file toolbar belongs to the Files section only; showing it above the
          Radar would imply the two are related. */}
      {section === "files" && (
        <header className="flex flex-wrap items-center gap-2 border-b border-gray-700 bg-gray-800 px-4 py-2">
          <div className="flex items-center gap-1 rounded-2xl bg-gray-700 p-1">
            {(["local", "serial"] as const).map((mode) => (
              <button
                key={mode}
                onClick={() => handleSwitchMode(mode)}
                className={`rounded-2xl px-3 py-1.5 text-sm font-semibold transition ${
                  viewMode === mode
                    ? "bg-emerald-600 text-white"
                    : "text-gray-200 hover:text-white"
                }`}
              >
                {mode === "local" ? t("files.mode.local") : t("files.mode.serial")}
              </button>
            ))}
          </div>

          {viewMode === "serial" && !isMobile && (
            <DevicePanel onConnectionChange={setSerialConnected} mockMode={mockMode} />
          )}

          <button
            onClick={dir.goUp}
            className="rounded-2xl border border-gray-600 bg-gray-700 px-3 py-1.5 text-sm text-gray-200 hover:bg-gray-600"
          >
            {t("files.up")}
          </button>
          <span className="flex-1 truncate rounded-2xl bg-gray-700 px-3 py-1.5 font-mono text-sm text-gray-300">
            {viewMode === "serial" && mockMode ? "/mock" : dir.currentPath || t("files.no_folder")}
          </span>

          <input
            type="text"
            placeholder={t("files.search")}
            value={dir.searchQuery}
            onChange={(e) => dir.setSearchQuery(e.target.value)}
            className="w-40 rounded-2xl border border-gray-600 bg-gray-700 px-3 py-1.5 text-sm text-gray-100 placeholder-gray-500 focus:border-emerald-500 focus:outline-none"
          />
          <button
            onClick={() => setShowNewFile(!showNewFile)}
            className="rounded-2xl border border-emerald-500 bg-emerald-600 px-3 py-1.5 text-sm font-semibold text-white hover:bg-emerald-500"
          >
            {t("files.new")}
          </button>
        </header>
      )}

      {section === "files" && dir.error && (
        <div
          role="alert"
          className="flex items-center justify-between gap-4 border-b border-red-700 bg-red-950 px-4 py-2 text-sm text-red-100"
        >
          <span className="truncate">{dir.error}</span>
          <button onClick={() => dir.setError(null)} className="text-red-300 hover:text-red-100">
            {t("common.close")}
          </button>
        </div>
      )}

      {showNewFile && section === "files" && (
        <NewFileModal
          currentPath={dir.currentPath}
          onClose={() => setShowNewFile(false)}
          onCreate={handleCreateFile}
        />
      )}

      {section === "radar" && (
        <RadarView
          connected={effectiveSerialConnected}
          mockMode={mockMode}
          onPerform={(signal, action) => void handleSignalAction(signal, action)}
        />
      )}

      {section === "files" && (
        <FilesView
          dir={dir}
          editor={editor}
          dnd={dnd}
          viewMode={viewMode}
          serialConnected={effectiveSerialConnected}
          onSelectFile={editor.openFile}
          onOpenDir={handleNavigate}
        />
      )}

      {section === "tools" && (
        <ToolsView
          currentPath={dir.currentPath}
          openFiles={editor.tabs.map((tab) => ({ name: tab.file.name, content: tab.content }))}
        />
      )}

      {section === "device" && (
        <DeviceView connected={serialConnected} onConnectionChange={setSerialConnected} />
      )}

      {section === "settings" && (
        <SettingsView mockMode={mockMode} onMockModeChange={setMockMode} />
      )}

      <ToastContainer />
    </AppShell>
  );
}
