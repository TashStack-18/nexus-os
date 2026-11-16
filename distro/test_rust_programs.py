#!/usr/bin/env python3
import subprocess
import os
import sys

base = "/root/micro-os/distro/theme/nexus-studio"
import importlib.util
spec = importlib.util.spec_from_loader("studio", importlib.machinery.SourceFileLoader("studio", base))
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

env = dict(os.environ)
env["PATH"] = f"/root/.cargo/bin:{env.get('PATH', '')}"

for prog in ["01_hello_system", "02_task_simulator"]:
    print(f"=== Testing real compilation and execution of {prog} ===")
    pdir = f"/tmp/test_rust/{prog}"
    os.makedirs(f"{pdir}/src", exist_ok=True)
    with open(f"{pdir}/Cargo.toml", "w") as f:
        f.write(mod.EXAMPLES[prog]["toml"])
    with open(f"{pdir}/src/main.rs", "w") as f:
        f.write(mod.EXAMPLES[prog]["main"])
    res = subprocess.run(["cargo", "run", "--quiet"], cwd=pdir, capture_output=True, text=True, env=env)
    if res.returncode != 0:
        print("FAILED:", res.stderr)
        sys.exit(1)
    else:
        print(res.stdout)
print("=== BOTH REAL RUST PROGRAMS COMPILED & EXECUTED SUCCESSFULLY ===")
