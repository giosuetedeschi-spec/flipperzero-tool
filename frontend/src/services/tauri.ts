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
