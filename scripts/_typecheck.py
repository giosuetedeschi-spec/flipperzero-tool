
import subprocess, os, sys
env = os.environ.copy()
env['PATH'] = r'C:\Users\gioma\.hermes\node;' + env.get('PATH', '')
result = subprocess.run(
    [r'C:\Users\gioma\.hermes\node\node.exe', 
     r'C:\Cose Nuove\Code\flipperzero-tool\frontend\node_modules\typescript\bin\tsc.js',
     '--noEmit'],
    capture_output=True, text=True, timeout=30, env=env,
    cwd=r'C:\Cose Nuove\Code\flipperzero-tool\frontend'
)
print(result.stdout)
if result.stderr:
    print("STDERR:", result.stderr[:500])
sys.exit(result.returncode)
