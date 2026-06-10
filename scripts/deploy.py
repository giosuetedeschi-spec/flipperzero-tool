#!/usr/bin/env python3
import paramiko, os, time

def log(msg):
    print(f'[{time.strftime(\'H:%M:%S\')}] {msg}')

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(\'192.168.1.12\', username=\'root\', password=\'root\', timeout=10)
log(\'Connected\')

# Clean up old files
client.exec_command(\'rm -f /root/flipperzero-tool/src-tauri/src/serial.rs /root/flipperzero-tool/src-tauri/src/serial.rs.bak\', timeout=5)
log(\'Cleaned old serial files\')

# Upload new files
sftp = client.open_sftp()
repo = r\'C:\Cose Nuove\Code\flipperzero-tool\'
files = [
    (repo + r\'\src-tauri\src\serial\mod.rs\', \'/root/flipperzero-tool/src-tauri/src/serial/mod.rs\'),
    (repo + r\'\src-tauri\src\commands.rs\', \'/root/flipperzero-tool/src-tauri/src/commands.rs\'),
    (repo + r\'\src-tauri\src\main.rs\', \'/root/flipperzero-tool/src-tauri/src/main.rs\'),
]
for local, remote in files:
    sftp.put(local, remote)
    log(f\'  {os.path.basename(local)}\')
sftp.close()

# cargo check
cmd = \'source /root/.cargo/env && export PATH=/root/.cargo/bin:$PATH && cargo check --manifest-path /root/flipperzero-tool/src-tauri/Cargo.toml 2>&1 | tail -15\'
_, out, _ = client.exec_command(cmd, timeout=180)
o = out.read().decode()
lines = o.strip().split(\'\n\')
errors = [l for l in lines if \'error[\' in l.lower()]
log(f\'Errors: {len(errors)}\')
if errors:
    for e in errors[:5]: log(f\'  {e}\')
else:
    log(\'SUCCESS!\')
for l in lines[-8:]: print(l)

client.close()
log(\'Done\')
