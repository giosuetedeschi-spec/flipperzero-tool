# Feature: Editor Avanzato

## Stato: COMPLETATO

## Descrizione

Editor multi-file con tabs, find/replace, auto-save, word wrap, line numbers.
Estende l'editor CodeMirror esistente con funzionalita' professionali.

## Funzionalita

1. **Multi-Tap**: apri piu file contemporaneamente, switch tra tab
2. **Find/Replace**: ricerca testuale con case-sensitive, navigazione match precedente/successivo
3. **Replace One/All**: sostituzione singola o globale
4. **Auto-Save**: salvataggio automatico 2 dopo ultima modifica
5. **Word Wrap**: toggle a capo automatico
6. **Line Numbers**: toggle numeri riga
7. **Dirty Indicator**: pallino arancione su tab modificati non salvati
8. **Save All**: salvataggio batch di tutti i tab dirty
9. **Close All**: chiusura batch di tutti i tab
10. **Status Bar**: righe, caratteri, stato modifica, modalita'

## Architettura

### Hook: useEditor.ts (riscritto)

Nuovo hook con stato centralizzato:

```
EditorState {
  tabs: Tab[]
  activeTabIndex: number
  search: SearchState
  showSearch: boolean
  autoSave: boolean
  wordWrap: boolean
  lineNumbers: boolean
}
```

API esportate:
- openFile(file), closeTab(index), setActiveTab(index), closeAll()
- updateContent(index, content), saveFile(index), saveAll()
- toggleSearch(), setSearchQuery(q), setReplace(r)
- toggleCaseSensitive(), findNext(), findPrev()
- replaceOne(), replaceAll()
- toggleAutoSave(), toggleWordWrap(), toggleLineNumbers()

### Componente: EditorPanel.tsx (riscritto)

Layout:
- Tab Bar: lista tab con icona tipo file, nome, indicatore dirty, pulsante chiudi
- Toolbar: pulsanti Find/Replace, Auto-Save, Wrap, Lines, Save
- Search Bar: campo find, campo replace, pulsanti navigazione, case-sensitive, Replace/All
- Editor Area: CodeMirror con props aggiornate
- Status Bar: info file, righe, caratteri, stato

### CodeMirror Editor (da aggiornare)

Nuove props attese:
- searchQuery: stringa di ricerca evidenziata
- searchCaseSensitive: boolean
- searchCurrentMatch: indice match corrente

## File Modificati

| File | Azione |
|------|--------|
| frontend/src/hooks/useEditor.ts | RISCRITTO - multi-tab, find/replace, auto-save |
| frontend/src/components/EditorPanel.tsx | RISCRITTO - tabs, toolbar, search bar, status bar |
| frontend/src/App.tsx | MODIFICATO - nuove props EditorPanel rimuovo FilePreview separato |

## Prossimi Passi

- Aggiornare CodeMirrorEditor per supportare search highlighting
- Aggiungere shortcut da tastiera (Ctrl+S, Ctrl+F, Ctrl+H)
- Aggiungere undo/redo history per tab
