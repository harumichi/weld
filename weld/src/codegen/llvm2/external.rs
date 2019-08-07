//! Managing external llvm module other than generated one.
//! Especially for functions which should be inlined in terms of performance.


use libc;
use llvm_sys;

use std::ffi::{CStr, CString};
use libc::c_char;

use std::collections::{HashMap, HashSet};

use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::bit_reader::*;
use llvm_sys::linker::*;
use llvm_sys::LLVMLinkage;

use crate::error::WeldResult;
use crate::codegen::llvm2::llvm_exts::*;


/// Link modules loaded from bitcodes into a module in first argument.
pub unsafe fn link_module_from_bitcode(
    dst_module: LLVMModuleRef, src_bitcodes: &Vec<String>
) -> WeldResult<LLVMModuleRef> {
    // TODO: dispose intermediate object when error
    let mut modules = vec![dst_module];
    let context = LLVMGetModuleContext(dst_module);
    for bc in src_bitcodes {
        let mut membuf = 0 as LLVMMemoryBufferRef;
        let mut msg = 0 as *mut c_char;

        let path = CString::new(bc.clone()).unwrap();
        let ret = LLVMCreateMemoryBufferWithContentsOfFile(
            path.as_ptr(), &mut membuf as *mut LLVMMemoryBufferRef, &mut msg as *mut *mut c_char
        );
        if ret != 0 {
            return compile_err!("create memory buffer for '{}' \n{}",
                                path.to_str().unwrap(),
                                CStr::from_ptr(msg).to_str().unwrap());
        }

        let mut module = 0 as LLVMModuleRef;
        let ret = LLVMParseBitcodeInContext2(
            context, membuf, &mut module as *mut LLVMModuleRef
        );
        if ret != 0 {
            return compile_err!("parse bitcode for {}", path.to_str().unwrap());
        }
        LLVMDisposeMemoryBuffer(membuf);
        modules.push(module);
    }

    link_module(modules)
}

/// Link modules given in argument. first module in vector is destination.
/// take care this destructing existing llvm valueref
pub unsafe fn link_module(mut modules: Vec<LLVMModuleRef>) -> WeldResult<LLVMModuleRef> {
    if modules.len() == 0 {
        return compile_err!("at least one module is needed to link.");
    }

    while modules.len() > 1 {
        let dst = modules[0];
        let src = modules.pop().unwrap();
        let ret = LLVMLinkModules2(dst, src);
        if ret != 0 {
            // TODO: dispose rest module
            return compile_err!("fail to link module");
        }
    }
    Ok(modules[0])
}

pub unsafe fn add_inline_attr_in(
    context: LLVMContextRef, module: LLVMModuleRef, funcnames: &Vec<String>
) -> WeldResult<()> {
    let attr = [LLVMExtAttribute::AlwaysInline];

    for name in funcnames {
        let c_name = CString::new(name.clone()).unwrap();
        let function = LLVMGetNamedFunction(module, c_name.as_ptr());
        // TODO: handle err in case function is not found
        LLVMExtAddAttrsOnFunction(context, function, &attr);
    }
    Ok(())
}

pub unsafe fn set_private_linkage_in(
    context: LLVMContextRef, module: LLVMModuleRef, funcnames: &Vec<String>
) -> WeldResult<()> {
    for name in funcnames {
        let c_name = CString::new(name.clone()).unwrap();
        let function = LLVMGetNamedFunction(module, c_name.as_ptr());
        // TODO: handle err in case function is not found
        LLVMSetLinkage(function, LLVMLinkage::LLVMPrivateLinkage);
    }
    Ok(())
}

pub mod sleef {
    use std::sync::Mutex;
    use llvm_sys::core::*;

    use super::*;
    use super::super::*;

    use crate::ast::ScalarKind::{self, F32, F64};
    use crate::ast::UnaryOpKind::{self, *};
    use crate::error::WeldResult;


    lazy_static! {
        pub static ref SLEEF_FUNC_INFOS: HashMap<UnaryOpKind, [&'static str; 2]> = {
            let mut hm = HashMap::new();

            hm.insert(Exp, ["exp", "u10"]);
            hm.insert(Log, ["log", "u10"]);
            hm.insert(Sqrt, ["sqrt", "u05"]);
            hm.insert(Sin, ["sin", "u10"]);
            hm.insert(Cos, ["cos", "u10"]);
            hm.insert(Tan, ["tan", "u10"]);
            hm.insert(ASin, ["asin", "u10"]);
            hm.insert(ACos, ["acos", "u10"]);
            hm.insert(ATan, ["atan", "u10"]);
            hm.insert(Sinh, ["sinh", "u10"]);
            hm.insert(Cosh, ["cosh", "u10"]);
            hm.insert(Tanh, ["tanh", "u10"]);
            hm.insert(Erf, ["erf", "u10"]);
            hm
        };

//        pub static ref SLEEF_CALLED_FUNCS: Mutex<HashSet<String>> = {
//            Mutex::new(HashSet::new())
//        };

        pub static ref SLEEF_ALL_FUNCS: Vec<String> = {
            let mut af = Vec::new();
            let scalars = vec![ScalarKind::F32, ScalarKind::F64];
            let widths = vec![LLVM_VECTOR_WIDTH];

            for op in SLEEF_FUNC_INFOS.keys() {
                for scalar in scalars.iter() {
                    for width in widths.iter() {
                        let fn_name = func_name(
                            op.clone(), scalar.clone(), Some(width.clone())
                        ).unwrap();
                        af.push(fn_name);
                    }
                }
            }
            af
        };

        pub static ref SLEEF_BITCODE_DIR: String = {
            use std::env;
            use std::path::Path;

            let weld_home = env::var("WELD_HOME").expect("WELD_HOME is not set");
            let path = format!(
                "{}/weld/sleef/sleef/build/lib",
                weld_home,
            );
            if ! Path::new(&path).exists() {
                panic!("directory having bitcode does not exist");
            }
            path
        };

        pub static ref SLEEF_BITCODES: Vec<String> = {
            let mut s = Vec::new();

            s.push("weldsimdsp_AVX2128.bc");
//            s.push("weldsimddp_AVX.bc");
            s.push("weldsimddp_FMA4.bc");
            s.push("rempitab.bc");

            let mut sb = Vec::new();
            for name in s.into_iter() {
                sb.push(format!("{}/{}", *SLEEF_BITCODE_DIR, name));
            }
            sb
        };
    }

    /// Return whether op is supported. None of simd width means scalar.
    pub fn support_op(op: UnaryOpKind, scalar: ScalarKind, simd_width: Option<u32>) -> bool {
        let mut ret = (*SLEEF_FUNC_INFOS).contains_key(&op);
        ret = ret && match scalar {
            F32 | F64 => true,
            _ => false
        };
        ret = ret && simd_width.is_some();
        ret
    }

    pub fn func_name(op: UnaryOpKind, scalar: ScalarKind, simd_width: Option<u32>)
        -> WeldResult<String> {
        if !support_op(op, scalar, simd_width) {
            return compile_err!("does not support op in sleef")
        }

        let info = (*SLEEF_FUNC_INFOS).get(&op).unwrap();
        let mut name = format!("Sleef_{}", info[0]);
        name.push_str(match scalar {
            F32 => "f",
            F64 => "d",
            _ => { unreachable!(); }
        });
        name.push_str(&format!("{}_", simd_width.unwrap()));
        name.push_str(&format!("{}", info[1]));
        name.push_str(match scalar {
            F32 => "avx2128",
//            F64 => "avx",
            F64 => "fma4",
            _ => { unreachable!(); }
        });

        Ok(name)
    }

    pub unsafe fn link_sleef_module(module: LLVMModuleRef)
        -> WeldResult<LLVMModuleRef> {
        return link_module_from_bitcode(module, &(*SLEEF_BITCODES));
    }

    pub unsafe fn inline_function(context: LLVMContextRef, module: LLVMModuleRef)
                                  -> WeldResult<()> {
        let mut funcs = Vec::new();
        for func in SLEEF_ALL_FUNCS.iter() {
            debug!("inlined: {} ", func);
            funcs.push((*func).clone());
        }
        add_inline_attr_in(context, module, &funcs)?;
        set_private_linkage_in(context, module, &funcs)?;
        Ok(())
    }

}
