cfg_if::cfg_if! {
    if #[cfg(feature = "gurobi12")] {
        pub use grb_sys2_12::*;

        pub mod shims {
            use super::*;
            pub unsafe fn empty_env(env: *mut *mut GRBenv) -> c_int{
                let (major, minor, technical) = crate::version();
                return GRBemptyenvinternal(env, major, minor, technical)
            }


            pub unsafe fn load_env(env: *mut *mut GRBenv, logfilename: c_str) -> c_int{
                let (major, minor, technical) = crate::version();
                return GRBloadenvinternal(env, logfilename, major, minor, technical)
            }
        }
    } else {
        pub use grb_sys2_10::*;

        pub mod shims {
            use super::*;

            pub unsafe fn empty_env(env: *mut *mut GRBenv) -> c_int{
                return GRBemptyenv(env)
            }


            pub unsafe fn load_env(env: *mut *mut GRBenv, logfilename: c_str) -> c_int{
                return GRBloadenv(env, logfilename)
            }
        }
    }
}
