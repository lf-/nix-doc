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

void forceLambda(Value & v, const Pos & pos)
{
#ifdef NIX_2_4_0
    if (v.type() != nFunction) {
        throwTypeError(pos, "%2%: value is %1% while a lambda was expected", v);
    }
#else
    if (v.type != tLambda) {
        throwTypeError("%2%: value is %1% while a lambda was expected", v, pos);
    }
#endif
}


/* Print documentation of the given lambda. */
void prim_getDoc(EvalState & state, const nix::Pos & pos, Value * * args, Value & v)
{
    /* ensure the argument is a function */
    state.forceValue(*args[0], pos);

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

#ifdef NIX_2_4_0
    state.mkPos(v, ptr{&args[0]->lambda.fun->pos});
#else
    state.mkPos(v, &args[0]->lambda.fun->pos);
#endif
}

static RegisterPrimOp rp1("__getDoc", 1, prim_getDoc);
static RegisterPrimOp rp2("__doc", 1, prim_printDoc);
static RegisterPrimOp rp3("__unsafeGetLambdaPos", 1, prim_unsafeGetLambdaPos);

