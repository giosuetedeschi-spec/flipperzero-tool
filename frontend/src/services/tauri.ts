import { invoke } from "@tauri-apps/api/core";

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  is_dir: boolean;
  modified: string | null;
}

export interface AppError {
  IoError?: string;
  NotFound?: string;
  PermissionDenied?: string;
  SerialError?: string;
  ParseError?: string;
  DbError?: string;
  AlreadyExists?: string;
  General?: string;
}

export interface PortInfo {
  name: string;
  port_type: string;
  description: string | null;
}

function getErrorMessage(error: AppError | string): string {
  if (typeof error === "string") return error;
  const entry = Object.entries(error)[0];
  return entry ? entry[1] || entry[0] : "Unknown error";
}

// ---------------------------------------------------------------------------
// Local filesystem
// ---------------------------------------------------------------------------

export async function listDirectory(path: string): Promise<FileInfo[]> {
  try { return await invoke<FileInfo[]>("list_directory", { path }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function moveFile(source: string, dest: string): Promise<void> {
  try { await invoke("move_file", { source, dest }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function findFiles(path: string, pattern: string): Promise<FileInfo[]> {
  try { return await invoke<FileInfo[]>("find_files", { path, pattern }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function createFileFromTemplate(path: string, ext: string): Promise<string> {
  try { return await invoke<string>("create_file_from_template", { path, ext }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

// ---------------------------------------------------------------------------
// CRUD commands (FASE 1)
// ---------------------------------------------------------------------------

export async function rename_file(path: string, new_name: string): Promise<string> {
  try { return await invoke<string>("rename_file", { path, new_name }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function delete_file(path: string): Promise<void> {
  try { await invoke("delete_file", { path }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function copy_file(source: string, dest: string): Promise<void> {
  try { await invoke("copy_file", { source, dest }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function get_file_content(path: string): Promise<string> {
  try { return await invoke<string>("get_file_content", { path }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function write_file_content(path: string, data: string): Promise<void> {
  try { await invoke("write_file_content", { path, data }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function get_app_paths(): Promise<{ home: string; desktop: string; documents: string }> {
  try {
    const result = await invoke<{ home: string; desktop: string; documents: string }>("get_app_paths");
    return result;
  } catch (error) {
    throw new Error(getErrorMessage(error as AppError | string));
  }
}

// ---------------------------------------------------------------------------
// Local file read/write
// ---------------------------------------------------------------------------

export async function localReadFile(path: string): Promise<string> {
  try { return await invoke<string>("local_read_file", { path }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function localWriteFile(path: string, data: string): Promise<boolean> {
  try { return await invoke<boolean>("local_write_file", { path, data }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

// ---------------------------------------------------------------------------
// Serial commands
// ---------------------------------------------------------------------------

export async function serialListPorts(): Promise<PortInfo[]> {
  try { return await invoke<PortInfo[]>("serial_list_ports"); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function serialConnect(port: string): Promise<boolean> {
  try { return await invoke<boolean>("serial_connect", { port }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function serialDisconnect(): Promise<boolean> {
  try { return await invoke<boolean>("serial_disconnect"); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function serialReadFile(path: string): Promise<string> {
  try { return await invoke<string>("serial_read_file", { path }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function serialWriteFile(path: string, data: string): Promise<boolean> {
  try { return await invoke<boolean>("serial_write_file", { path, data }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function serialListDir(path: string): Promise<FileInfo[]> {
  try { return await invoke<FileInfo[]>("serial_list_dir", { path }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function serialIsConnected(): Promise<boolean> {
  try { return await invoke<boolean>("serial_is_connected"); }
  catch { return false; }
}

// uFBT commands (FASE 4)
export async function ufbt_is_installed(): Promise<boolean> {
  try { return await invoke<boolean>("ufbt_is_installed"); } catch (e) { throw new Error(getErrorMessage(e)); }
}
export async function ufbt_get_version(): Promise<string> {
  try { return await invoke<string>("ufbt_get_version"); } catch (e) { throw new Error(getErrorMessage(e)); }
}
export async function ufbt_get_sdk_version(): Promise<string> {
  try { return await invoke<string>("ufbt_get_sdk_version"); } catch (e) { throw new Error(getErrorMessage(e)); }
}
export async function ufbt_install(): Promise<string> {
  try { return await invoke<string>("ufbt_install"); } catch (e) { throw new Error(getErrorMessage(e)); }
}
export async function ufbt_update(): Promise<string> {
  try { return await invoke<string>("ufbt_update"); } catch (e) { throw new Error(getErrorMessage(e)); }
}
export async function ufbt_create(name: string, path: string): Promise<string> {
  try { return await invoke<string>("ufbt_create", { name, path }); } catch (e) { throw new Error(getErrorMessage(e)); }
}
export async function ufbt_build(path: string): Promise<string> {
  try { return await invoke<string>("ufbt_build", { path }); } catch (e) { throw new Error(getErrorMessage(e)); }
}
export async function ufbt_deploy(path: string): Promise<string> {
  try { return await invoke<string>("ufbt_deploy", { path }); } catch (e) { throw new Error(getErrorMessage(e)); }
}
export async function ufbt_clean(path: string): Promise<string> {
  try { return await invoke<string>("ufbt_clean", { path }); } catch (e) { throw new Error(getErrorMessage(e)); }
}


// ---------------------------------------------------------------------------
// Serial extended commands (P2)
// ---------------------------------------------------------------------------

export async function serialDelete(path: string): Promise<boolean> {
  return invoke<boolean>("serial_delete", { path });
}

export async function serialMkdir(path: string): Promise<boolean> {
  return invoke<boolean>("serial_mkdir", { path });
}

export async function serialStat(path: string): Promise<FileInfo> {
  return invoke<FileInfo>("serial_stat", { path });
}

export async function serialAutodetectConnect(): Promise<boolean> {
  return invoke<boolean>("serial_autodetect_connect");
}

export async function serialUpload(localPath: string, remotePath: string): Promise<boolean> {
  return invoke<boolean>("serial_upload", { localPath, remotePath });
}

export async function serialDownload(remotePath: string, localPath: string): Promise<boolean> {
  return invoke<boolean>("serial_download", { remotePath, localPath });
}

// ---------------------------------------------------------------------------
// VFS commands (P2)
// ---------------------------------------------------------------------------

export async function fsClearCache(): Promise<void> {
  return invoke<void>("fs_clear_cache");
}

export async function fsRemoveFile(path: string): Promise<void> {
  return invoke<void>("fs_remove_file", { path });
}

export interface ReindexProgress {
  current: number;
  total: number;
  path: string;
}

// ---------------------------------------------------------------------------
// Upload/Download with progress callback (P2d)
// ---------------------------------------------------------------------------

export interface TransferProgress {
  bytesTransferred: number;
  totalBytes: number;
  fileName: string;
}

// ---------------------------------------------------------------------------
// Structured parser commands (P3)
// ---------------------------------------------------------------------------

export async function parserParseSubStruct(data: string): Promise<any> {
  return invoke<any>("parser_parse_sub_struct", { data });
}

export async function parserParseIrStruct(data: string): Promise<any> {
  return invoke<any>("parser_parse_ir_struct", { data });
}

export async function parserParseNfcStruct(data: string): Promise<any> {
  return invoke<any>("parser_parse_nfc_struct", { data });
}

// ---------------------------------------------------------------------------
// Template commands (P3)
// ---------------------------------------------------------------------------

export async function templateGet(ext: string): Promise<string> {
  return invoke<string>("template_get", { ext });
}

export async function templateList(): Promise<string[]> {
  return invoke<string[]>("template_list");
}

export async function templateCreate(basePath: string, name: string, ext: string): Promise<string> {
  return invoke<string>("template_create", { basePath, name, ext });
}


// ---------------------------------------------------------------------------
// Reverse Engineer
// ---------------------------------------------------------------------------

export interface PatternMatch {
  offset: number;
  length: number;
  pattern: number[];
  confidence: number;
  description: string;
}

export interface ProtocolFingerprint {
  name: string;
  signature: number[];
  offset: number;
  description: string;
}

export interface FieldCandidate {
  offset: number;
  length: number;
  field_type: string;
  confidence: number;
  value_hex: string;
  value_dec: number | null;
}

export interface AnalysisResult {
  entropy: number;
  total_bytes: number;
  unique_bytes: number;
  patterns: PatternMatch[];
  matched_protocols: ProtocolFingerprint[];
  inferred_structure: FieldCandidate[];
  hex_preview: string;
  ascii_preview: string;
}

export async function reverseEngineerAnalyze(hexData: string): Promise<AnalysisResult> {
  try { return await invoke<AnalysisResult>("reverse_engineer_analyze", { hexData }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}

export async function reverseEngineerAnalyzeFile(path: string): Promise<AnalysisResult> {
  try { return await invoke<AnalysisResult>("reverse_engineer_analyze_file", { path }); }
  catch (error) { throw new Error(getErrorMessage(error as AppError | string)); }
}
