import { useState } from 'react';
import { ufbt_create, ufbt_build, ufbt_deploy, ufbt_clean } from '../services/tauri';
import { showToast } from './ui/Toast';

interface Project { name: string; path: string; }

export default function FapProjectManager() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [newName, setNewName] = useState('');
  const [newPath, setNewPath] = useState('');
  const [busy, setBusy] = useState<string | null>(null);

  const handleCreate = async () => {
    if (!newName.trim() || !newPath.trim()) { showToast('Name and path required', 'error'); return; }
    setBusy('create');
    try {
      await ufbt_create(newName.trim(), newPath.trim());
      setProjects([...projects, { name: newName.trim(), path: newPath.trim() }]);
      setNewName(''); setNewPath('');
      showToast('Created ' + newName.trim(), 'success');
    } catch (err) { showToast(err instanceof Error ? err.message : String(err), 'error'); }
    finally { setBusy(null); }
  };

  const handleBuild = async (p: Project) => {
    setBusy('build-' + p.name);
    try { const o = await ufbt_build(p.path); showToast('Build: ' + o.slice(0, 80), 'success'); }
    catch (err) { showToast(err instanceof Error ? err.message : String(err), 'error'); }
    finally { setBusy(null); }
  };

  const handleDeploy = async (p: Project) => {
    setBusy('deploy-' + p.name);
    try { const o = await ufbt_deploy(p.path); showToast('Deploy: ' + o.slice(0, 80), 'success'); }
    catch (err) { showToast(err instanceof Error ? err.message : String(err), 'error'); }
    finally { setBusy(null); }
  };

  const handleClean = async (p: Project) => {
    setBusy('clean-' + p.name);
    try { await ufbt_clean(p.path); showToast('Cleaned', 'success'); }
    catch (err) { showToast(err instanceof Error ? err.message : String(err), 'error'); }
    finally { setBusy(null); }
  };

  return (
    <div className='bg-gray-800 border border-gray-700 rounded-lg p-4 space-y-3'>
      <h3 className='text-sm font-bold text-gray-300'>FAP Projects</h3>
      <div className='space-y-2'>
        <input type='text' placeholder='Project name' value={newName}
          onChange={e => setNewName(e.target.value)}
          className='w-full px-2 py-1 bg-gray-700 rounded text-xs placeholder-gray-500' />
        <input type='text' placeholder='Project path' value={newPath}
          onChange={e => setNewPath(e.target.value)}
          className='w-full px-2 py-1 bg-gray-700 rounded text-xs placeholder-gray-500' />
        <button onClick={handleCreate} disabled={busy === 'create'}
          className='w-full px-3 py-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 rounded text-xs font-medium'>
          {busy === 'create' ? 'Creating...' : 'Create Project'}
        </button>
      </div>
      {projects.length === 0 ? (
        <p className='text-xs text-gray-500 italic'>No projects yet</p>
      ) : (
        <div className='space-y-2'>
          {projects.map(p => (
            <div key={p.name} className='bg-gray-700 rounded p-2 space-y-1'>
              <div className='text-xs font-medium text-gray-300'>{p.name}</div>
              <div className='text-xs text-gray-500 font-mono truncate'>{p.path}</div>
              <div className='flex gap-1'>
                <button onClick={() => handleBuild(p)} disabled={busy !== null}
                  className='px-2 py-0.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-40 rounded text-xs'>Build</button>
                <button onClick={() => handleDeploy(p)} disabled={busy !== null}
                  className='px-2 py-0.5 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 rounded text-xs'>Deploy</button>
                <button onClick={() => handleClean(p)} disabled={busy !== null}
                  className='px-2 py-0.5 bg-gray-600 hover:bg-gray-500 disabled:opacity-40 rounded text-xs'>Clean</button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
