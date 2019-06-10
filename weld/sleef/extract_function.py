
import json
import os
import subprocess


def get_targets():
    bitcode_funcs = {
        # double
        "sleefsimddp_AVX.bc": [
            "Sleef_expd4_u10avx",
            "Sleef_logd4_u10avx",            
            "Sleef_sqrtd4_u05avx",       
            "Sleef_sind4_u10avx",       
            "Sleef_cosd4_u10avx",       
            "Sleef_tand4_u10avx",       
            "Sleef_asind4_u10avx",       
            "Sleef_acosd4_u10avx",       
            "Sleef_atand4_u10avx",       
            "Sleef_sinhd4_u10avx",       
            "Sleef_coshd4_u10avx",       
            "Sleef_tanhd4_u10avx",       
            "Sleef_erfd4_u10avx",       
        ],
        "sleefsimddp_FMA4.bc": [
            "Sleef_expd4_u10fma4",
            "Sleef_logd4_u10fma4",            
            "Sleef_sqrtd4_u05fma4",       
            "Sleef_sind4_u10fma4",       
            "Sleef_cosd4_u10fma4",       
            "Sleef_tand4_u10fma4",       
            "Sleef_asind4_u10fma4",       
            "Sleef_acosd4_u10fma4",       
            "Sleef_atand4_u10fma4",       
            "Sleef_sinhd4_u10fma4",       
            "Sleef_coshd4_u10fma4",       
            "Sleef_tanhd4_u10fma4",       
            "Sleef_erfd4_u10fma4",       
        ],

        # float
        "sleefsimdsp_AVX2128.bc": [
            "Sleef_expf4_u10avx2128",
            "Sleef_logf4_u10avx2128",
            "Sleef_sqrtf4_u05avx2128",       
            "Sleef_sinf4_u10avx2128",       
            "Sleef_cosf4_u10avx2128",       
            "Sleef_tanf4_u10avx2128",       
            "Sleef_asinf4_u10avx2128",       
            "Sleef_acosf4_u10avx2128",       
            "Sleef_atanf4_u10avx2128",       
            "Sleef_sinhf4_u10avx2128",       
            "Sleef_coshf4_u10avx2128",       
            "Sleef_tanhf4_u10avx2128",       
            "Sleef_erff4_u10avx2128",
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
