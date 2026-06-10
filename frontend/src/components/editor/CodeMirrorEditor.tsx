import { useEffect, useRef, useCallback } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter, drawSelection } from "@codemirror/view";
import { defaultKeymap, indentWithTab, history, historyKeymap } from "@codemirror/commands";
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, foldGutter } from "@codemirror/language";
import { json } from "@codemirror/lang-json";
import { javascript } from "@codemirror/lang-javascript";
import { oneDark } from "@codemirror/theme-one-dark";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, completionKeymap } from "@codemirror/autocomplete";
import { lintKeymap } from "@codemirror/lint";

interface Props {
  value: string;
  onChange: (value: string) => void;
  language?: string;
  readOnly?: boolean;
}

function getLanguageExtension(lang?: string) {
  switch (lang) {
    case "json": return json();
    case "javascript":
    case "js":
    case "ts":
    case "typescript": return javascript();
    default: return [];
  }
}

export default function CodeMirrorEditor({ value, onChange, language, readOnly }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  // Initialize editor once
  useEffect(() => {
    if (!containerRef.current) return;

    const languageExt = getLanguageExtension(language);

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        onChangeRef.current(update.state.doc.toString());
      }
    });

    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLine(),
        highlightActiveLineGutter(),
        drawSelection(),
        history(),
        foldGutter(),
        bracketMatching(),
        indentWithTab,
        autocompletion(),
        highlightSelectionMatches(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        oneDark,
        languageExt,
        keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap, ...completionKeymap, ...lintKeymap]),
        updateListener,
        EditorView.theme({
          "&": { height: "100%", fontSize: "13px" },
          ".cm-scroller": { overflow: "auto" },
          ".cm-content": { fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace" },
        }),
        ...(readOnly ? [EditorState.readOnly.of(true)] : []),
      ],
    });

    const view = new EditorView({
      state,
      parent: containerRef.current,
    });

    viewRef.current = view;

    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [language, readOnly]);

  // Sync external value changes
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const currentValue = view.state.doc.toString();
    if (currentValue !== value) {
      view.dispatch({
        changes: { from: 0, to: currentValue.length, insert: value },
      });
    }
  }, [value]);

  return <div ref={containerRef} className="h-full w-full overflow-hidden" />;
}
