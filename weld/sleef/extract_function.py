
import json
import os
import subprocess


def get_targets():
    bitcode_funcs = {
        "sleefsimddp_AVX.bc": [
            "Sleef_expd4_u10avx"
        ],
        "sleefsimdsp_AVX2128.bc": [
            "Sleef_expf4_u10avx2128"
        ]
    }
    return bitcode_funcs


def extract_function():
    libdir = "{}/sleef/build/lib".format(
        os.path.dirname(os.path.abspath(__file__))
    )
    os.chdir(libdir)

    llvm_extract_bin = "{}/llvm-extract".format(
        subprocess.check_output(
            "llvm-config --bindir".split()
        ).strip()
    )
    llvm_dis_bin = "{}/llvm-dis".format(
        subprocess.check_output(
            "llvm-config --bindir".split()
        ).strip()
    )

    targets = get_targets()
    for bitcode, funcs in targets.items():
        func_flag = ""
        for fn in funcs:
            func_flag += "-func {} ".format(fn)
            
        dst = bitcode.replace("sleef", "weld") 
        cmd = "{binary} {src} -o {dst} {flag}".format(
            binary=llvm_extract_bin,
            src=bitcode,
            dst=dst,
            flag=func_flag
        )
        res = subprocess.call(cmd.split())

        cmd = "{binary} {src}".format(
            binary=llvm_dis_bin,
            src=dst
        )
        res = subprocess.call(cmd.split())


if __name__ == "__main__":
    extract_function()
