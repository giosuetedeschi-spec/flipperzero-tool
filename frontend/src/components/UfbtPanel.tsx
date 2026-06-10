import { useState, useEffect } from 'react';
import { ufbt_is_installed, ufbt_get_version, ufbt_get_sdk_version, ufbt_install, ufbt_update } from '../services/tauri';
import { showToast } from './ui/Toast';

export default function UfbtPanel() {
  const [installed, setInstalled] = useState(false);
  const [version, setVersion] = useState('');
  const [sdkVersion, setSdkVersion] = useState('');
  const [busy, setBusy] = useState(false);

  useEffect(() => { checkUfbt(); }, []);

  const checkUfbt = async () => {
    try {
      const isInstalled = await ufbt_is_installed();
      setInstalled(isInstalled);
      if (isInstalled) {
        const v = await ufbt_get_version();
        setVersion(v);
        try { const sdkV = await ufbt_get_sdk_version(); setSdkVersion(sdkV); } catch { /* ok */ }
      }
    } catch (err) {
      showToast('Failed: ' + (err instanceof Error ? err.message : String(err)), 'error');
    }
  };

  const handleInstall = async () => {
    setBusy(true);
    try { await ufbt_install(); showToast('Installed', 'success'); await checkUfbt(); }
    catch (err) { showToast('Failed: ' + (err instanceof Error ? err.message : String(err)), 'error'); }
    finally { setBusy(false); }
  };

  const handleUpdate = async () => {
    setBusy(true);
    try { await ufbt_update(); showToast('Updated', 'success'); await checkUfbt(); }
    catch (err) { showToast('Failed: ' + (err instanceof Error ? err.message : String(err)), 'error'); }
    finally { setBusy(false); }
  };

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-4 space-y-3">
      <h3 className="text-sm font-bold text-gray-300">uFBT Tool</h3>
      <div className="flex items-center gap-2 text-xs">
        <span className={"w-2 h-2 rounded-full " + (installed ? "bg-emerald-400" : "bg-red-400")} />
        <span className="text-gray-400">{installed ? 'Installed' : 'Not installed'}</span>
      </div>
      {version && <div className="text-xs text-gray-500">Version: {version}</div>}
      {sdkVersion && <div className="text-xs text-gray-500">SDK: {sdkVersion}</div>}
      <div className="flex gap-2">
        {!installed ? (
          <button onClick={handleInstall} disabled={busy}
            className="px-3 py-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 rounded text-xs font-medium">
            {busy ? 'Installing...' : 'Install uFBT'}
          </button>
        ) : (
          <button onClick={handleUpdate} disabled={busy}
            className="px-3 py-1 bg-blue-600 hover:bg-blue-500 disabled:opacity-40 rounded text-xs font-medium">
            {busy ? 'Updating...' : 'Update'}
          </button>
        )}
        <button onClick={checkUfbt} disabled={busy}
          className="px-3 py-1 bg-gray-700 hover:bg-gray-600 disabled:opacity-40 rounded text-xs">
          Refresh
        </button>
      </div>
    </div>
  );
}
