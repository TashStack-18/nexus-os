import importlib.machinery

loader = importlib.machinery.SourceFileLoader('lunara_vitals', '/root/micro-os/distro/theme/lunara-vitals')
mod = loader.load_module()
win = mod.LunaraVitalsWindow()
win.show_all()
print("SUCCESS: Vitals window created! Stats: CPU=", win.stats.cpu_pct, "RAM=", win.stats.mem_used_mb, "Kernel=", win.stats.kernel_version)
