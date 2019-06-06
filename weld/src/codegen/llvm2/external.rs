

use libc;
use llvm_sys;

use std::ffi::{CStr, CString};

use libc::c_char;

use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::bit_reader::*;
use llvm_sys::linker::*;
use llvm_sys::LLVMLinkage;
use crate::error::WeldResult;


/// link modules loaded from bitcodes into that in first argument.
pub unsafe fn link_module_from_bitcode(dst_module: LLVMModuleRef, src_bitcodes: Vec<String>)
    -> WeldResult<LLVMModuleRef> {
    let mut modules = vec![dst_module];
    for bc in src_bitcodes {
        let mut membuf = 0 as LLVMMemoryBufferRef;
        let mut msg = 0 as *mut c_char;

        let path = CString::new(bc.clone()).unwrap().as_ptr();
        let ret = LLVMCreateMemoryBufferWithContentsOfFile(
            path, &mut membuf as *mut LLVMMemoryBufferRef, &mut msg as *mut *mut c_char
        );
        if ret != 0 {
            return compile_err!("create memory buffer for {}", bc);
        }

        let mut module = 0 as LLVMModuleRef;
        let ret = LLVMParseBitcode(
            membuf, &mut module as *mut LLVMModuleRef,
            &mut msg as *mut *mut c_char
        );
        if ret != 0 {
            return compile_err!("parse bitcode for {}", bc);
        }
        LLVMDisposeMemoryBuffer(membuf);
        modules.push(module);
    }

    link_module(modules)
}


/// link modules given in argument. first module in vector is destination.
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


