#![allow(non_camel_case_types)]
#![allow(improper_ctypes)]

pub use std::os::raw::{c_char, c_double, c_int, c_void};
pub type c_str = *const c_char;

#[repr(C)]
pub struct GRBenv;

#[repr(C)]
pub struct GRBmodel;

#[repr(C)]
pub struct GRBsvec {
    /// sparse vector length
    pub len: c_int,
    /// indices array of the sparse vector
    pub ind: *mut c_int,
    /// value array of the sparse vector
    pub val: *mut c_double,
}

// Environment Creation and Destruction
extern "C" {
    pub fn GRBemptyenv(envP: *mut *mut GRBenv) -> c_int;

    pub fn GRBstartenv(envP: *mut GRBenv) -> c_int;

    pub fn GRBloadenv(envP: *mut *mut GRBenv, logfilename: c_str) -> c_int;

    pub fn GRBgetmultiobjenv(model: *mut GRBmodel, num: c_int) -> *mut GRBenv;

    pub fn GRBdiscardmultiobjenvs(model: *mut GRBmodel);

    pub fn GRBfreeenv(env: *mut GRBenv);

    pub fn GRBgetconcurrentenv(model: *mut GRBmodel, num: c_int) -> *mut GRBenv;

    pub fn GRBdiscardconcurrentenvs(model: *mut GRBmodel);
}

// Model Creation and Modification
extern "C" {
    pub fn GRBnewmodel(
        env: *mut GRBenv,
        modelP: *mut *mut GRBmodel,
        Pname: c_str,
        numvars: c_int,
        obj: *const c_double,
        lb: *const c_double,
        ub: *const c_double,
        vtype: *const c_char,
        varnames: *const c_str,
    ) -> c_int;

    pub fn GRBcopymodel(model: *mut GRBmodel) -> *mut GRBmodel;

    pub fn GRBaddconstr(
        model: *mut GRBmodel,
        numnz: c_int,
        cind: *const c_int,
        cval: *const c_double,
        sense: c_char,
        rhs: c_double,
        constrname: c_str,
    ) -> c_int;

    pub fn GRBaddconstrs(
        model: *mut GRBmodel,
        numconstrs: c_int,
        numnz: c_int,
        cbeg: *const c_int,
        cind: *const c_int,
        cval: *const c_double,
        sense: *const c_char,
        rhs: *const c_double,
        constrname: *const c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrMax(
        model: *mut GRBmodel,
        name: c_str,
        resvar: c_int,
        nvars: c_int,
        vars: *const c_int,
        constant: c_double,
    ) -> c_int;

    pub fn GRBaddgenconstrMin(
        model: *mut GRBmodel,
        name: c_str,
        resvar: c_int,
        nvars: c_int,
        vars: *const c_int,
        constant: c_double,
    ) -> c_int;

    pub fn GRBaddgenconstrAbs(
        model: *mut GRBmodel,
        name: c_str,
        resvar: c_int,
        argvar: c_int,
    ) -> c_int;

    pub fn GRBaddgenconstrAnd(
        model: *mut GRBmodel,
        name: c_str,
        resvar: c_int,
        nvars: c_int,
        vars: *const c_int,
    ) -> c_int;

    pub fn GRBaddgenconstrOr(
        model: *mut GRBmodel,
        name: c_str,
        resvar: c_int,
        nvars: c_int,
        vars: *const c_int,
    ) -> c_int;

    pub fn GRBaddgenconstrNorm(
        model: *mut GRBmodel,
        name: c_str,
        resvar: c_int,
        nvars: c_int,
        vars: *const c_int,
        which: c_double,
    ) -> c_int;

    pub fn GRBaddgenconstrIndicator(
        model: *mut GRBmodel,
        name: c_str,
        binvar: c_int,
        binval: c_int,
        nvars: c_int,
        ind: *const c_int,
        val: *const c_double,
        sense: c_char,
        rhs: c_double,
    ) -> c_int;

    pub fn GRBaddgenconstrPWL(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        npts: c_int,
        xpts: *const c_double,
        ypts: *const c_double,
    ) -> c_int;

    pub fn GRBaddgenconstrPoly(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        plen: c_int,
        p: *mut c_double,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrExp(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrExpA(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        a: c_double,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrLog(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrLogA(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        a: c_double,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrLogistic(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrPow(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        a: c_double,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrSin(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrCos(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddgenconstrTan(
        model: *mut GRBmodel,
        name: c_str,
        xvar: c_int,
        yvar: c_int,
        options: c_str,
    ) -> c_int;

    pub fn GRBaddqconstr(
        model: *mut GRBmodel,
        numlnz: c_int,
        lind: *const c_int,
        lval: *const c_double,
        numqnz: c_int,
        qrow: *const c_int,
        qcol: *const c_int,
        qval: *const c_double,
        sense: c_char,
        rhs: c_double,
        QCname: c_str,
    ) -> c_int;

    pub fn GRBaddqpterms(
        model: *mut GRBmodel,
        numqnz: c_int,
        qrow: *const c_int,
        qcol: *const c_int,
        qval: *const c_double,
    ) -> c_int;

    pub fn GRBaddrangeconstr(
        model: *mut GRBmodel,
        numnz: c_int,
        cind: *const c_int,
        cval: *const c_double,
        lower: c_double,
        upper: c_double,
        constrname: c_str,
    ) -> c_int;

    pub fn GRBaddrangeconstrs(
        model: *mut GRBmodel,
        numconstrs: c_int,
        numnz: c_int,
        cbeg: *const c_int,
        cind: *const c_int,
        cval: *const c_double,
        lower: *const c_double,
        upper: *const c_double,
        constrname: *const c_str,
    ) -> c_int;

    pub fn GRBaddsos(
        model: *mut GRBmodel,
        numsos: c_int,
        nummembers: c_int,
        types: *const c_int,
        beg: *const c_int,
        ind: *const c_int,
        weight: *const c_double,
    ) -> c_int;

    pub fn GRBaddvar(
        model: *mut GRBmodel,
        numnz: c_int,
        vind: *const c_int,
        vval: *const c_double,
        obj: f64,
        lb: f64,
        ub: f64,
        vtype: c_char,
        name: c_str,
    ) -> c_int;

    pub fn GRBaddvars(
        model: *mut GRBmodel,
        numvars: c_int,
        numnz: c_int,
        vbeg: *const c_int,
        vind: *const c_int,
        vval: *const c_double,
        obj: *const f64,
        lb: *const f64,
        ub: *const f64,
        vtype: *const c_char,
        name: *const c_str,
    ) -> c_int;

    pub fn GRBchgcoeffs(
        model: *mut GRBmodel,
        cnt: c_int,
        cind: *const c_int,
        vind: *const c_int,
        val: *const c_double,
    ) -> c_int;

    pub fn GRBdelvars(model: *mut GRBmodel, numdel: c_int, ind: *const c_int) -> c_int;

    pub fn GRBdelconstrs(model: *mut GRBmodel, numdel: c_int, ind: *const c_int) -> c_int;

    pub fn GRBdelgenconstrs(model: *mut GRBmodel, numdel: c_int, ind: *const c_int) -> c_int;

    pub fn GRBdelq(model: *mut GRBmodel) -> c_int;

    pub fn GRBdelqconstrs(model: *mut GRBmodel, len: c_int, ind: *const c_int) -> c_int;

    pub fn GRBdelsos(model: *mut GRBmodel, len: c_int, ind: *const c_int) -> c_int;

    pub fn GRBsetpwlobj(
        model: *mut GRBmodel,
        var: c_int,
        points: c_int,
        x: *const c_double,
        y: *const c_double,
    ) -> c_int;

    pub fn GRBupdatemodel(model: *mut GRBmodel) -> c_int;

    pub fn GRBfreemodel(model: *mut GRBmodel) -> c_int;
}

// Model Solution
extern "C" {

    pub fn GRBoptimize(model: *mut GRBmodel) -> c_int;

    pub fn GRBoptimizeasync(model: *mut GRBmodel) -> c_int;

    pub fn GRBcomputeIIS(model: *mut GRBmodel) -> c_int;

    pub fn GRBfeasrelax(
        model: *mut GRBmodel,
        relaxobjtype: c_int,
        minrelax: c_int,
        lbpen: *const c_double,
        ubpen: *const c_double,
        rhspen: *const c_double,
        feasobjP: *const c_double,
    ) -> c_int;

    pub fn GRBfixmodel(model: *mut GRBmodel, new_model: *mut *mut GRBmodel) -> c_int;

    pub fn GRBresetmodel(model: *mut GRBmodel) -> c_int;

    pub fn GRBsync(model: *mut GRBmodel) -> c_int;
}

// Model Queries
extern "C" {
    pub fn GRBgetcoeff(
        model: *mut GRBmodel,
        constr: c_int,
        var: c_int,
        valP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetconstrbyname(model: *mut GRBmodel, name: c_str, constrnumP: *mut c_int) -> c_int;

    pub fn GRBgetconstrs(
        model: *mut GRBmodel,
        numnzP: *mut c_int,
        cbeg: *mut c_int,
        cind: *mut c_int,
        cval: *mut c_double,
        start: c_int,
        len: c_int,
    ) -> c_int;

    pub fn GRBgetenv(model: *mut GRBmodel) -> *mut GRBenv;

    pub fn GRBgetgenconstrMax(
        model: *mut GRBmodel,
        id: c_int,
        resvarP: *mut c_int,
        nvarsP: *mut c_int,
        vars: *mut c_int,
        constantP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrMin(
        model: *mut GRBmodel,
        id: c_int,
        resvarP: *mut c_int,
        nvarsP: *mut c_int,
        vars: *mut c_int,
        constantP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrAbs(
        model: *mut GRBmodel,
        id: c_int,
        resvarP: *mut c_int,
        argvarP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetgenconstrAnd(
        model: *mut GRBmodel,
        id: c_int,
        resvarP: *mut c_int,
        nvarsP: *mut c_int,
        vars: *mut c_int,
    ) -> c_int;

    pub fn GRBgetgenconstrOr(
        model: *mut GRBmodel,
        id: c_int,
        resvarP: *mut c_int,
        nvarsP: *mut c_int,
        vars: *mut c_int,
    ) -> c_int;

    pub fn GRBgetgenconstrNorm(
        model: *mut GRBmodel,
        id: c_int,
        resvarP: *mut c_int,
        nvarsP: *mut c_int,
        vars: *mut c_int,
        whichP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrIndicator(
        model: *mut GRBmodel,
        id: c_int,
        binvarP: *mut c_int,
        binvalP: *mut c_int,
        nvarsP: *mut c_int,
        ind: *mut c_int,
        val: *mut c_double,
        senseP: *mut c_char,
        rhsP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrPWL(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
        nptsP: *mut c_int,
        xpts: *mut c_double,
        y_pts: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrPoly(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
        plenP: *mut c_int,
        p: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrExp(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetgenconstrExpA(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
        aP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrLog(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetgenconstrLogA(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
        aP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrLogistic(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetgenconstrPow(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
        aP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetgenconstrSin(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetgenconstrCos(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetgenconstrTan(
        model: *mut GRBmodel,
        id: c_int,
        xvarP: *mut c_int,
        yvarP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetpwlobj(
        model: *mut GRBmodel,
        var: c_int,
        npointsP: *mut c_int,
        x: *mut c_double,
        y: *mut c_double,
    ) -> c_int;

    pub fn GRBgetq(
        model: *mut GRBmodel,
        numqnzP: *mut c_int,
        qrow: *mut c_int,
        qcol: *mut c_int,
        qval: *mut c_double,
    ) -> c_int;

    pub fn GRBgetqconstr(
        model: *mut GRBmodel,
        qconstr: c_int,
        numlnzP: *mut c_int,
        lind: *mut c_int,
        lval: *mut c_double,
        numqnzP: *mut c_int,
        qrow: *mut c_int,
        qcol: *mut c_int,
        qval: *mut c_double,
    ) -> c_int;

    pub fn GRBgetsos(
        model: *mut GRBmodel,
        nummembersP: *mut c_int,
        sostype: *mut c_int,
        beg: *mut c_int,
        ind: *mut c_int,
        weight: *mut c_double,
        start: c_int,
        len: c_int,
    ) -> c_int;

    pub fn GRBgetvarbyname(model: *mut GRBmodel, name: c_str, varnumP: *mut c_int) -> c_int;

    pub fn GRBgetvars(
        model: *mut GRBmodel,
        numnzP: *mut c_int,
        vbeg: *mut c_int,
        vind: *mut c_int,
        vval: *mut c_double,
        start: c_int,
        len: c_int,
    ) -> c_int;

    pub fn GRBsinglescenariomodel(
        model: *mut GRBmodel,
        singlescenarioP: *mut *mut GRBmodel,
    ) -> c_int;

    // Xgetconstrs
    // Xgetvars
}

// Input/Output
extern "C" {
    pub fn GRBreadmodel(env: *mut GRBenv, filename: c_str, modelP: *mut *mut GRBmodel) -> c_int;

    pub fn GRBread(model: *mut GRBmodel, filename: c_str) -> c_int;

    pub fn GRBwrite(model: *mut GRBmodel, filename: c_str) -> c_int;

}

extern "C" {
    pub fn GRBgetattrinfo(
        model: *mut GRBmodel,
        attrname: c_str,
        datatypeP: *mut c_int,
        attrtypeP: *mut c_int,
        settableP: *mut c_int,
    ) -> c_int;
}

extern "C" {
    pub fn GRBgetintattr(model: *mut GRBmodel, attrname: c_str, valueP: *mut c_int) -> c_int;

    pub fn GRBgetdblattr(model: *mut GRBmodel, attrname: c_str, valueP: *mut c_double) -> c_int;

    pub fn GRBgetstrattr(model: *mut GRBmodel, attrname: c_str, valueP: *mut c_str) -> c_int;

    pub fn GRBsetintattr(model: *mut GRBmodel, attrname: c_str, value: c_int) -> c_int;

    pub fn GRBsetdblattr(model: *mut GRBmodel, attrname: c_str, value: c_double) -> c_int;

    pub fn GRBsetstrattr(model: *mut GRBmodel, attrname: c_str, value: c_str) -> c_int;
}

extern "C" {
    pub fn GRBgetintattrelement(
        model: *mut GRBmodel,
        attrname: c_str,
        element: c_int,
        valueP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetdblattrelement(
        model: *mut GRBmodel,
        attrname: c_str,
        element: c_int,
        valueP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetcharattrelement(
        model: *mut GRBmodel,
        attrname: c_str,
        element: c_int,
        valueP: *mut c_char,
    ) -> c_int;

    pub fn GRBgetstrattrelement(
        model: *mut GRBmodel,
        attrname: c_str,
        element: c_int,
        valueP: *mut c_str,
    ) -> c_int;

    pub fn GRBsetintattrelement(
        model: *mut GRBmodel,
        attrname: c_str,
        element: c_int,
        value: c_int,
    ) -> c_int;

    pub fn GRBsetdblattrelement(
        model: *mut GRBmodel,
        attrname: c_str,
        element: c_int,
        value: c_double,
    ) -> c_int;

    pub fn GRBsetcharattrelement(
        model: *mut GRBmodel,
        attrname: c_str,
        element: c_int,
        value: c_char,
    ) -> c_int;

    pub fn GRBsetstrattrelement(
        model: *mut GRBmodel,
        attrname: c_str,
        element: c_int,
        value: c_str,
    ) -> c_int;
}

extern "C" {
    pub fn GRBgetintattrarray(
        model: *mut GRBmodel,
        attrname: c_str,
        first: c_int,
        len: c_int,
        values: *mut c_int,
    ) -> c_int;

    pub fn GRBgetdblattrarray(
        model: *mut GRBmodel,
        attrname: c_str,
        first: c_int,
        len: c_int,
        values: *mut c_double,
    ) -> c_int;

    pub fn GRBgetcharattrarray(
        model: *mut GRBmodel,
        attrname: c_str,
        first: c_int,
        len: c_int,
        values: *mut c_char,
    ) -> c_int;

    pub fn GRBgetstrattrarray(
        model: *mut GRBmodel,
        attrname: c_str,
        first: c_int,
        len: c_int,
        values: *mut c_str,
    ) -> c_int;

    pub fn GRBsetintattrarray(
        model: *mut GRBmodel,
        attrname: c_str,
        first: c_int,
        len: c_int,
        values: *const c_int,
    ) -> c_int;

    pub fn GRBsetdblattrarray(
        model: *mut GRBmodel,
        attrname: c_str,
        first: c_int,
        len: c_int,
        values: *const c_double,
    ) -> c_int;

    pub fn GRBsetcharattrarray(
        model: *mut GRBmodel,
        attrname: c_str,
        first: c_int,
        len: c_int,
        values: *const c_char,
    ) -> c_int;

    pub fn GRBsetstrattrarray(
        model: *mut GRBmodel,
        attrname: *const c_char,
        first: c_int,
        len: c_int,
        values: *const c_str,
    ) -> c_int;
}

extern "C" {
    pub fn GRBgetintattrlist(
        model: *mut GRBmodel,
        attrname: c_str,
        len: c_int,
        ind: *const c_int,
        values: *mut c_int,
    ) -> c_int;

    pub fn GRBgetdblattrlist(
        model: *mut GRBmodel,
        attrname: c_str,
        len: c_int,
        ind: *const c_int,
        values: *mut c_double,
    ) -> c_int;

    pub fn GRBgetcharattrlist(
        model: *mut GRBmodel,
        attrname: c_str,
        len: c_int,
        ind: *const c_int,
        values: *mut c_char,
    ) -> c_int;

    pub fn GRBgetstrattrlist(
        model: *mut GRBmodel,
        attrname: c_str,
        len: c_int,
        ind: *const c_int,
        values: *mut c_str,
    ) -> c_int;

    pub fn GRBsetintattrlist(
        model: *mut GRBmodel,
        attrname: c_str,
        len: c_int,
        ind: *const c_int,
        values: *const c_int,
    ) -> c_int;

    pub fn GRBsetdblattrlist(
        model: *mut GRBmodel,
        attrname: c_str,
        len: c_int,
        ind: *const c_int,
        values: *const c_double,
    ) -> c_int;

    pub fn GRBsetcharattrlist(
        model: *mut GRBmodel,
        attrname: c_str,
        len: c_int,
        ind: *const c_int,
        values: *const c_char,
    ) -> c_int;

    pub fn GRBsetstrattrlist(
        model: *mut GRBmodel,
        attrname: *const c_char,
        len: c_int,
        ind: *const c_int,
        values: *const c_str,
    ) -> c_int;
}

// Parameter Management and Tuning
extern "C" {
    pub fn GRBtunemodel(model: *mut GRBmodel) -> c_int;

    pub fn GRBgettuneresult(model: *mut GRBmodel, n: c_int) -> c_int;

    pub fn GRBgetdblparam(env: *mut GRBenv, paramname: c_str, value: *mut c_double) -> c_int;

    pub fn GRBgetintparam(env: *mut GRBenv, paramname: c_str, value: *mut c_int) -> c_int;

    pub fn GRBgetstrparam(env: *mut GRBenv, paramname: c_str, value: *mut c_char) -> c_int;

    pub fn GRBsetdblparam(env: *mut GRBenv, paramname: c_str, value: c_double) -> c_int;

    pub fn GRBsetintparam(env: *mut GRBenv, paramname: c_str, value: c_int) -> c_int;

    pub fn GRBsetstrparam(env: *mut GRBenv, paramname: c_str, value: c_str) -> c_int;

    pub fn GRBgetdblparaminfo(
        env: *mut GRBenv,
        paramname: c_str,
        valueP: *mut c_double,
        minP: *mut c_double,
        maxP: *mut c_double,
        defaultP: *mut c_double,
    ) -> c_int;

    pub fn GRBgetintparaminfo(
        env: *mut GRBenv,
        paramname: c_str,
        valueP: *mut c_int,
        minP: *mut c_int,
        maxP: *mut c_int,
        defaultP: *mut c_int,
    ) -> c_int;

    pub fn GRBgetstrparaminfo(
        env: *mut GRBenv,
        paramname: c_str,
        valueP: *mut c_char,
        defaultP: *mut c_char,
    ) -> c_int;

    pub fn GRBreadparams(env: *mut GRBenv, filename: c_str) -> c_int;

    pub fn GRBwriteparams(env: *mut GRBenv, filename: c_str) -> c_int;
}

// Monitoring Progress - Logging and Callbacks
extern "C" {
    pub fn GRBmsg(env: *mut GRBenv, message: c_str);

    pub fn GRBsetcallbackfunc(
        model: *mut GRBmodel,
        cb: Option<extern "C" fn(*mut GRBmodel, *mut c_void, c_int, *mut c_void) -> c_int>,
        usrdata: *mut c_void,
    ) -> c_int;

    pub fn GRBgetcallbackfunc(
        model: *mut GRBmodel,
        cb: *mut Option<extern "C" fn(*mut GRBmodel, *mut c_void, c_int, *mut c_void) -> c_int>,
    ) -> c_int;

    pub fn GRBcbget(cbdata: *mut c_void, where_: c_int, what: c_int, resultP: *mut c_void)
        -> c_int;

    pub fn GRBversion(majorP: *mut c_int, minorP: *mut c_int, technicalP: *mut c_int);
}

// Modifying Solver Behaviour - Callbacks
extern "C" {
    pub fn GRBcbcut(
        cbdata: *mut c_void,
        cutlen: c_int,
        cutind: *const c_int,
        cutval: *const c_double,
        cutsense: c_char,
        cutrhs: c_double,
    ) -> c_int;

    pub fn GRBcblazy(
        cbdata: *mut c_void,
        lazylen: c_int,
        lazyind: *const c_int,
        lazyval: *const c_double,
        lazysense: c_char,
        lazyrhs: c_double,
    ) -> c_int;

    pub fn GRBcbsolution(
        cbdata: *mut c_void,
        solution: *const c_double,
        obj_ptr: *mut c_double,
    ) -> c_int;

    pub fn GRBterminate(model: *mut GRBmodel);

    pub fn GRBcbproceed(model: *mut GRBmodel);
}

// Error Handling
extern "C" {
    pub fn GRBgeterrormsg(env: *mut GRBenv) -> c_str;
}

// Advanced simplex routines
extern "C" {
    pub fn GRBFSolve(model: *mut GRBmodel, b: *mut GRBsvec, x: *mut GRBsvec) -> c_int;

    pub fn GRBBSolve(model: *mut GRBmodel, b: *mut GRBsvec, x: *mut GRBsvec) -> c_int;

    pub fn GRBBinvColj(model: *mut GRBmodel, j: c_int, x: *mut GRBsvec) -> c_int;

    pub fn GRBBinvRowi(model: *mut GRBmodel, i: c_int, x: *mut GRBsvec) -> c_int;

    pub fn GRBgetBasisHead(model: *mut GRBmodel, bhead: *mut c_int) -> c_int;
}

// cheeky hack to get unit tests for build scripts.
#[cfg(feature = "build_script_tests")]
#[path = "../build.rs"]
#[allow(unused)]
mod build;
