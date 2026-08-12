import FileTable from "../components/FileTable";
import EditorPanel from "../components/EditorPanel";
import FileInspector from "../components/FileInspector";
import type { FileInfo } from "../services/tauri";
import type { useDirectory } from "../hooks/useDirectory";
import type { useEditor } from "../hooks/useEditor";
import type { useDragDrop } from "../hooks/useDragDrop";
import { useT } from "../i18n/useT";
import { useIsMobileLayout } from "../shell/useBreakpoint";

/**
 * The file browser and editor, lifted out of App.tsx unchanged.
 *
 * On desktop it keeps the original two-pane layout. On a phone the panes stack:
 * the browser until a file is opened, then the editor, because two panes side by
 * side on a 390px screen leaves neither usable.
 *
 * A file that a dedicated viewer understands gets that viewer above the editor,
 * which is how the Sub-GHz, NFC and IR analysers finally become reachable.
 */

interface Props {
  dir: ReturnType<typeof useDirectory>;
  editor: ReturnType<typeof useEditor>;
  dnd: ReturnType<typeof useDragDrop>;
  viewMode: "local" | "serial";
  serialConnected: boolean;
  onSelectFile: (file: FileInfo) => void;
  onOpenDir: (file: FileInfo) => void;
}

export default function FilesView({
  dir,
  editor,
  dnd,
  viewMode,
  serialConnected,
  onSelectFile,
  onOpenDir,
}: Props) {
  const { t } = useT();
  const isMobile = useIsMobileLayout();
  const activeTab = editor.tabs[editor.activeTabIndex];
  const hasOpenFile = activeTab !== undefined;

  const browser = (
    <div className="flex-1 overflow-auto">
      {viewMode === "serial" && !serialConnected ? (
        <div className="flex h-full flex-col items-center justify-center gap-3 text-gray-500">
          <span className="text-4xl">🔌</span>
          <span>{t("files.connect_first")}</span>
        </div>
      ) : dir.loading ? (
        <div className="flex h-full items-center justify-center text-gray-500">
          <span className="mr-2 animate-spin">⏳</span> {t("files.loading")}
        </div>
      ) : dir.files.length === 0 ? (
        <div className="flex h-full items-center justify-center text-gray-500">
          {dir.searchQuery ? t("files.no_match") : t("files.empty")}
        </div>
      ) : (
        <FileTable
          files={dir.files}
          selectedPath={editor.selectedFile?.path ?? null}
          onSelect={onSelectFile}
          onOpen={onOpenDir}
          onDragStart={dnd.handleDragStart}
          onDropOnDir={dnd.handleDropOnDir}
        />
      )}
    </div>
  );

  const inspectorAndEditor = (
    <div className="flex flex-1 flex-col overflow-auto">
      {activeTab && (
        <FileInspector
          file={editor.selectedFile}
          content={activeTab.content}
          onClose={() => editor.closeTab(editor.activeTabIndex)}
        />
      )}
      <EditorPanel
        tabs={editor.tabs}
        activeTabIndex={editor.activeTabIndex}
        showSearch={editor.showSearch}
        search={editor.search}
        autoSave={editor.autoSave}
        wordWrap={editor.wordWrap}
        lineNumbers={editor.lineNumbers}
        hasDirtyTabs={editor.hasDirtyTabs}
        dirtyCount={editor.dirtyCount}
        onContentChange={editor.updateContent}
        onSave={editor.saveFile}
        onSaveAll={editor.saveAll}
        onClose={editor.closeTab}
        onCloseAll={editor.closeAll}
        onSetActive={editor.setActiveTab}
        onToggleSearch={editor.toggleSearch}
        onSetSearchQuery={editor.setSearchQuery}
        onSetReplace={editor.setReplace}
        onToggleCaseSensitive={editor.toggleCaseSensitive}
        onFindNext={editor.findNext}
        onFindPrev={editor.findPrev}
        onReplaceOne={editor.replaceOne}
        onReplaceAll={editor.replaceAll}
        onToggleAutoSave={editor.toggleAutoSave}
        onToggleWordWrap={editor.toggleWordWrap}
        onToggleLineNumbers={editor.toggleLineNumbers}
        viewMode={viewMode}
      />
    </div>
  );

  // Stacked on a phone: whichever pane is relevant right now gets the screen.
  if (isMobile) {
    return (
      <div className="flex flex-1 flex-col overflow-hidden" data-testid="files-view">
        {hasOpenFile ? inspectorAndEditor : browser}
      </div>
    );
  }

  return (
    <div className="flex flex-1 overflow-hidden" data-testid="files-view">
      {browser}
      {inspectorAndEditor}
    </div>
  );
}
