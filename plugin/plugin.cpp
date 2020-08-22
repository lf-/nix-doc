#include <config.h>
#include <primops.hh>
#include <globals.hh>
#include <eval-inline.hh>
#include <dlfcn.h>
#include <iostream>

#if HAVE_BOEHMGC

#include <gc/gc.h>
#include <gc/gc_cpp.h>

#endif

#ifdef NIX_3_0_0
#define throwTypeError_ throwTypeError
#define throwTypeError(msg, val, pos) throwTypeError_(pos, msg, val)
#endif

using namespace nix;

extern "C" {
char const * nd_get_function_docs(char const * filename, size_t line, size_t col);
void nd_free_string(char const * str);
}

/* Print documentation of the given lambda. */
void prim_getDoc(EvalState & state, const nix::Pos & pos, Value * * args, Value & v)
{
    /* ensure the argument is a function */
    state.forceValue(*args[0], pos);
    if (args[0]->type != tLambda) {
        throwTypeError("%2%: value is %1% while a lambda was expected", *args[0], pos);
    }

    auto poz = args[0]->lambda.fun->pos;
    std::string const & file = poz.file;
    char const * doc = nd_get_function_docs(file.c_str(), poz.line, poz.column);
    if (doc == nullptr) {
        mkNull(v);
    } else {
        // this copies the string
        mkString(v, doc);
        nd_free_string(doc);
    }
}

void printLambdaDocs(Value & v)
{
    auto poz = v.lambda.fun->pos;
    std::string const & file = poz.file;
    char const * doc = nd_get_function_docs(file.c_str(), poz.line, poz.column);
    if (doc != nullptr) {
        std::cout << doc << std::endl;
        nd_free_string(doc);
    }

}

void forceLambda(Value & v, const Pos & pos)
{
    if (v.type != tLambda) {
        throwTypeError("%2%: value is %1% while a lambda was expected", v, pos);
    }
}

/* Return documentation of the given lambda. */
void prim_printDoc(EvalState & state, const Pos & pos, Value * * args, Value & v)
{
    /* ensure the argument is a function */
    state.forceValue(*args[0], pos);
    forceLambda(*args[0], pos);

    printLambdaDocs(*args[0]);
    mkNull(v);
}

/* Return position information of the given lambda. */
void prim_unsafeGetLambdaPos(EvalState & state, const Pos & pos, Value * * args, Value & v)
{
    /* ensure the argument is a function */
    state.forceValue(*args[0], pos);
    forceLambda(*args[0], pos);

    state.mkPos(v, &args[0]->lambda.fun->pos);
}

static RegisterPrimOp rp1("__getDoc", 1, prim_getDoc);
static RegisterPrimOp rp2("__doc", 1, prim_printDoc);
static RegisterPrimOp rp3("__unsafeGetLambdaPos", 1, prim_unsafeGetLambdaPos);

// only include the :doc repl command if our nix supports it
// https://github.com/NixOS/nix/pull/3934
#if __has_include("repl.hh")
#include "repl.hh"

void replCmd(NixRepl & repl, string, string arg)
{
    Value v;
    repl.evalString(arg, v);
    // it has been evaluated now
    // TODO: (pending nix-doc hacking), support non-function attributes
    // We actually have all the info to use unsafeGetAttrPos (more or less) with
    // nice syntax, but I don't think nix-doc can get docs for attributes
    forceLambda(v, noPos);
    printLambdaDocs(v);
}

static RegisterReplCmd rc1({"doc"}, "Get the `nix-doc` documentation for <expr>", replCmd, "<expr>");
#endif
