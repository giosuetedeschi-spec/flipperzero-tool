#!/usr/bin/env python3
import os, sys, time
import paramiko

REPO = r"C:\Cose Nuove\Code\flipperzero-tool"
TAR = r"C:\Users\gioma\Desktop\flipper.tar.gz"
HOST = "192.168.1.12"
CARGO = "/root/.cargo/bin/cargo"
RUSTUP = "source /root/.cargo/env"

def log(msg):
    ts = time.strftime("%H:%M:%S")
    print(f"[{ts}] {msg}")

def create_tar():
    log("Creating tar...")
    import tarfile
    skip = {"node_modules", "target", ".git", "__pycache__", ".github", "scripts"}
    with tarfile.open(TAR, 'w:gz') as tar:
        for root, dirs, files in os.walk(REPO):
            dirs[:] = [d for d in dirs if d not in skip]
            for f in files:
                full = os.path.join(root, f)
                rel = os.path.relpath(full, os.path.dirname(REPO))
                tar.add(full, arcname=rel)
    size = os.path.getsize(TAR)
    log(f"Tar: {size/1024:.0f} KB")

def run():
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    client.connect(HOST, username="root", password="root", timeout=10)
    log("Connected")
    sftp = client.open_sftp()
    sftp.put(TAR, "/root/flipper.tar.gz")
    sftp.close()
    log("Uploaded")
    c = client.exec_command
    # Extract to /tmp first, then move to avoid double nesting
    c("cd /tmp && tar -xzf /root/flipper.tar.gz && mv flipperzero-tool /root/flipperzero-tool 2>/dev/null; rm -rf /root/flipperzero-tool && mv /tmp/flipperzero-tool /root/ 2>/dev/null; ls /root/flipperzero-tool/src-tauri/Cargo.toml", timeout=30)
    log("Extracted")

    cargo_cmd = f'{RUSTUP} && cd /root/flipperzero-tool'

    # cargo check
    log("Running cargo check...")
    _, out, err = c(f"{cargo_cmd} && {CARGO} check 2>&1", timeout=180)
    o, e = out.read().decode(), err.read().decode()
    print('\n=== CARGO CHECK (last 40 lines) ===')
    lines_out = o.strip().split('\n')
    print('\n'.join(lines_out[-40:]))
    if e.strip():
        err_lines = e.strip().split('\n')
        print('STDERR:', '\n'.join(err_lines[-20:]))

    # cargo test
    log("Running cargo test...")
    _, out, _ = c(f"{cargo_cmd} && {CARGO} test --lib 2>&1", timeout=180)
    print('\n=== CARGO TEST (last 40 lines) ===')
    lines_t = out.read().decode().strip().split('\n')
    print('\n'.join(lines_t[-40:]))

    client.close()
    log("Done!")

if __name__ == "__main__":
    create_tar()
    run()
