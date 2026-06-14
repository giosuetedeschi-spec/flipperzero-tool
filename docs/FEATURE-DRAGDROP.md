# Feature: Drag & Drop

## Stato: COMPLETATO

## Descrizione

Sistema drag and drop per spostare file e cartelle nell'interfaccia del file browser.
Permette di trascinare file su directory di destinazione per spostarli.

## Architettura

### Hook: useDragDrop.ts

Hook React custom che gestisce il ciclo di vita del drag and drop:

useDragDrop(onDrop: (file: FileInfo, targetPath: string) => void)

API:
- isDragging: boolean -- stato del drag in corso
- handleDragStart(file, e) -- inizia il drag di un file
- handleDragOver(e) -- gestisce il passaggio sopra una drop zone
- handleDragEnd() -- termina il drag
- handleDropOnDir(targetDir, e) -- gestisce il drop su una directory

Meccanismo:
1. handleDragStart serializza il file in dataTransfer come JSON
2. handleDropOnDir deserializza il file e chiama onDrop(file, targetPath)
3. L'handler onDrop nel componente chiama moveFile() del backend

### Componente: FileTable.tsx

Tabella file con supporto drag and drop:
- Ogni riga e' draggable
- Le directory accettano drop (onDragOver + onDrop)
- Le file non-directory sono solo drag source

### Integrazione: App.tsx

const dnd = useDragDrop(async (file, targetPath) => {
  const dest = targetPath + "/" + file.name;
  await moveFile(file.path, dest);
  dir.refresh();
  showToast("Moved " + file.name, "success");
});

### Backend: commands.rs

#[tauri::command]
pub fn move_file(source: String, dest: String) -> Result<(), AppError> {
    fs::rename(&source, &dest).map_err(AppError::from)?;
    Ok(())
}

## File Modificati

| File | Modifica |
|------|----------|
| frontend/src/App.tsx | Aggiunto import useDragDrop e moveFile, inizializzazione hook, props drag su FileTable |
| frontend/src/hooks/useDragDrop.ts | Gia esistente, nessuna modifica |
| frontend/src/components/FileTable.tsx | Gia esistente con props onDragStart e onDropOnDir |
| frontend/src/services/tauri.ts | Gia esistente con moveFile() |
| src-tauri/src/commands.rs | Gia esistente con move_file command |

## Testing

Per testare:
1. Apri il file browser (modalita locale o serial)
2. Trascina un file su una directory
3. Il file viene spostato e la lista si aggiorna
4. Toast di conferma appare

## Limitazioni

- Non supporta drag di cartelle (solo file)
- Non supporta drag multiplo
- Non supporta copia (solo spostamento)
