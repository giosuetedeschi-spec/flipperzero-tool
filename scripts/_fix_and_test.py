#!/usr/bin/env python3
import paramiko, os, time

def log(msg):
    print(f"[{time.strftime(\'H:%M:%S\')}] {msg}")

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(\'192.168.1.12\', username=\'root\', password=\'root\', timeout=10)
log(\'Connected\')

# Create dirs
client.exec_command(\'mkdir -p /root/flipperzero-tool/src-tauri/src/serial\', timeout=5)

# Upload files
sftp = client.open_sftp()
files = [
    (r\'C:\Cose Nuove\Code\flipperzero-tool\src-tauri\src\serial\mod.rs\', \'/root/flipperzero-tool/src-tauri/src/serial/mod.rs\'),
    (r\'C:\Cose Nuove\Code\flipperzero-tool\src-tauri\src\commands.rs\', \'/root/flipperzero-tool/src-tauri/src/commands.rs\'),
    (r\'C:\Cose Nuove\Code\flipperzero-tool\src-tauri\src\main.rs\', \'/root/flipperzero-tool/src-tauri/src/main.rs\'),
]
for local, remote in files:
    sftp.put(local, remote)
    log(f\'  {os.path.basename(local)}\')
sftp.close()

# cargo check
cmd = \'source /root/.cargo/env && export PATH=/root/.cargo/bin:$PATH && /root/.cargo/bin/cargo check --manifest-path /root/flipperzero-tool/src-tauri/Cargo.toml 2>&1\'
_, out, _ = client.exec_command(cmd, timeout=180)
o = out.read().decode()
lines = o.strip().split(\'\\n\')
errors = [l for l in lines if \'error[\' in l.lower()]
log(f\'Errors: {len(errors)}\')
if errors:
    for e in errors[:5]: log(f\'  {e}\')
else:
    log(\'SUCCESS!\')
for l in lines[-5:]: print(l)

client.close()
log(\'Done\')
