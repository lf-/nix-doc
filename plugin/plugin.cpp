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

using namespace nix;

extern "C" {
char const * nd_get_function_docs(char const * filename, size_t line, size_t col);
void nd_free_string(char const * str);
}

/* Return position information of the given lambda. */
void prim_getDoc(EvalState & state, const nix::Pos & pos, Value * * args, Value & v)
{
    /* ensure the argument is a function */
    state.forceValue(*args[0], pos);
    if (args[0]->type != tLambda) {
        throwTypeError("value is %1% while a lambda was expected", *args[0], pos);
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

/* Return position information of the given lambda. */
void prim_printDoc(EvalState & state, const nix::Pos & pos, Value * * args, Value & v)
{
    /* ensure the argument is a function */
    state.forceValue(*args[0], pos);
    if (args[0]->type != tLambda) {
        throwTypeError("value is %1% while a lambda was expected", *args[0], pos);
    }

    auto poz = args[0]->lambda.fun->pos;
    std::string const & file = poz.file;
    char const * doc = nd_get_function_docs(file.c_str(), poz.line, poz.column);
    mkNull(v);
    if (doc != nullptr) {
        std::cout << doc << std::endl;
        nd_free_string(doc);
    }
}

static RegisterPrimOp rp1("__getDoc", 1, prim_getDoc);
static RegisterPrimOp rp2("__doc", 1, prim_printDoc);
