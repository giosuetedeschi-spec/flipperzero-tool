
import subprocess, os, sys
env = os.environ.copy()
env['PATH'] = r'C:\Users\gioma\.hermes\node;' + env.get('PATH', '')
result = subprocess.run(
    ['bash', '-c', 'npm install'],
    capture_output=True, text=True, timeout=120, env=env,
    cwd=r'C:\Cose Nuove\Code\flipperzero-tool\frontend'
)
print(result.stdout[-1000:])
if result.stderr:
    print("STDERR:", result.stderr[-500:])
sys.exit(result.returncode)
