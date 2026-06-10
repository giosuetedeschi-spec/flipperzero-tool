#!/usr/bin/env python3
import paramiko

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect('192.168.1.12', username='root', password='root', timeout=10)

client.exec_command('mkdir -p /root/flipperzero-tool/frontend/dist', timeout=5)
client.exec_command('echo "<!DOCTYPE html><html><body>FlipperZero Tool</body></html>" > /root/flipperzero-tool/frontend/dist/index.html', timeout=5)

_, out, _ = client.exec_command('ls -la /root/flipperzero-tool/frontend/dist/', timeout=5)
print("dist:", out.read().decode().strip())

cmd = 'source /root/.cargo/env && export PATH=/root/.cargo/bin:$PATH && /root/.cargo/bin/cargo check --manifest-path /root/flipperzero-tool/src-tauri/Cargo.toml 2>&1'
_, out, _ = client.exec_command(cmd, timeout=180)
o = out.read().decode()
lines = o.strip().split('\n')
errors = [l for l in lines if 'error[' in l.lower()]
print(f'\nErrors: {len(errors)}')
if errors:
    for e in errors[:10]: print(e)
print('\n--- Last 15 ---')
for l in lines[-15:]: print(l)

client.close()
