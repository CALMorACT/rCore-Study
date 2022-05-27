
import os
# 用于将所有的 用户程序 编译出来
base_address = 0x80400000
step = 0x20000
linker = '../src/linker.ld'
target_dir = "../target/riscv64gc-unknown-none-elf/release/"

app_id = 0
apps = os.listdir('../src/bin')
apps.sort()
for app in apps:
    app = app[:app.find('.')]
    lines = []
    lines_before = []
    with open(linker, 'r') as f:
        for line in f.readlines():
            lines_before.append(line)
            line = line.replace(hex(base_address), hex(base_address+step*app_id))
            lines.append(line)
    with open(linker, 'w+') as f:
        f.writelines(lines)
    os.system('cargo build --bin %s --release --target=riscv64gc-unknown-none-elf' % app)
    os.system('rust-objcopy --strip-all %s -O binary %s.bin' % (target_dir+app, target_dir+app))
    print('[build.py] application %s start with address %s' %(app, hex(base_address+step*app_id)))
    with open(linker, 'w+') as f:
        f.writelines(lines_before)
    app_id = app_id + 1
