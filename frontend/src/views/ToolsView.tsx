import { useState } from "react";
import ReverseEngineerPanel from "../components/ReverseEngineerPanel";
import UfbtPanel from "../components/UfbtPanel";
import FapProjectManager from "../components/FapProjectManager";
import FileDiffViewer from "../components/FileDiffViewer";
import { useT } from "../i18n/useT";
import { useIsMobileLayout } from "../shell/useBreakpoint";

/**
 * Developer tools: byte-level analysis, capture comparison, and FAP building.
 *
 * All four panels existed already and none was reachable. The uFBT ones are
 * shown on a phone but explicitly disabled rather than hidden: the port promises
 * desktop parity, and a silently missing feature is worse than a stated
 * limitation -- the user is left wondering whether they broke something.
 */

interface OpenFile {
  name: string;
  content: string;
}

interface Props {
  currentPath: string;
  /** Files currently open in the editor, offered as comparison candidates. */
  openFiles: OpenFile[];
}

export default function ToolsView({ currentPath, openFiles }: Props) {
  const { t } = useT();
  const isMobile = useIsMobileLayout();
  const [leftIndex, setLeftIndex] = useState(0);
  const [rightIndex, setRightIndex] = useState(1);

  const canCompare = openFiles.length >= 2;
  const left = openFiles[leftIndex];
  const right = openFiles[rightIndex];

  return (
    <div className="flex flex-col gap-6 overflow-auto p-4" data-testid="tools-view">
      <section>
        <h2 className="mb-2 text-lg font-semibold text-white">{t("tools.analyze")}</h2>
        <ReverseEngineerPanel currentPath={currentPath} />
      </section>

      <section>
        <h2 className="mb-2 text-lg font-semibold text-white">{t("tools.compare")}</h2>
        {!canCompare ? (
          <p data-testid="compare-needs-two" className="rounded-lg bg-gray-800 p-3 text-sm text-gray-400">
            {t("tools.compare.needs_two")}
          </p>
        ) : (
          <>
            <div className="mb-3 flex flex-wrap gap-2">
              <select
                aria-label={t("tools.compare.left")}
                value={leftIndex}
                onChange={(e) => setLeftIndex(Number(e.target.value))}
                className="rounded-lg border border-gray-600 bg-gray-800 px-3 py-1.5 text-sm text-gray-100"
              >
                {openFiles.map((file, index) => (
                  <option key={file.name} value={index}>
                    {file.name}
                  </option>
                ))}
              </select>
              <select
                aria-label={t("tools.compare.right")}
                value={rightIndex}
                onChange={(e) => setRightIndex(Number(e.target.value))}
                className="rounded-lg border border-gray-600 bg-gray-800 px-3 py-1.5 text-sm text-gray-100"
              >
                {openFiles.map((file, index) => (
                  <option key={file.name} value={index}>
                    {file.name}
                  </option>
                ))}
              </select>
            </div>
            <FileDiffViewer
              contentA={left?.content ?? ""}
              contentB={right?.content ?? ""}
              fileNameA={left?.name ?? ""}
              fileNameB={right?.name ?? ""}
            />
          </>
        )}
      </section>

      <section>
        <h2 className="mb-2 text-lg font-semibold text-white">{t("tools.build")}</h2>
        {isMobile ? (
          <p
            data-testid="ufbt-desktop-only"
            className="rounded-lg border border-amber-700 bg-amber-950 p-3 text-sm text-amber-200"
          >
            {t("tools.build.desktop_only")}
          </p>
        ) : (
          <div className="flex flex-col gap-4">
            <UfbtPanel />
            <FapProjectManager />
          </div>
        )}
      </section>
    </div>
  );
}
