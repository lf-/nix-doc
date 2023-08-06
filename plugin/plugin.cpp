#include <config.h>
#include <dlfcn.h>
#include <eval-inline.hh>
#include <globals.hh>
#include <iostream>
#include <primops.hh>

#if HAVE_BOEHMGC

#include <gc/gc.h>
#include <gc/gc_cpp.h>

#endif

#include "compat.h"

using namespace nix;

extern "C" void discourage_linker_from_discarding() {}

extern "C" {
char const *nd_get_function_docs(char const *filename, size_t line, size_t col);
void nd_free_string(char const *str);
}

struct NdString {
  const char *docs;
  NdString(const char *docs) { this->docs = docs; }
  bool is_null() { return this->docs == nullptr; }
  ~NdString() {
    if (!is_null()) {
      nd_free_string(this->docs);
    }
  }
};

NdString docs_for_pos(nix::Pos const &pos) {
  std::string const file = compat::fileForPos(pos);
  return NdString{nd_get_function_docs(file.c_str(), pos.line, pos.column)};
}

/* Print documentation of the given lambda. */
void prim_getDoc(EvalState &state, compat::ConstPos pos, Value **args,
                 Value &v) {
  /* ensure the argument is a function */
  state.forceValue(*args[0], pos);
  compat::forceLambda(state, *args[0], pos);

  auto poz = compat::posForLambda(state, *args[0]->lambda.fun);
  auto doc = docs_for_pos(poz);
  if (doc.is_null()) {
    compat::mkNull(v);
  } else {
    // this copies the string
    compat::mkString(v, doc.docs);
  }
}

void printLambdaDocs(EvalState &state, Value &v) {
  auto poz = compat::posForLambda(state, *v.lambda.fun);

  auto docs = docs_for_pos(poz);
  if (!docs.is_null()) {
    std::cout << docs.docs << std::endl;
  }
}

/* Return documentation of the given lambda. */
void prim_printDoc(EvalState &state, compat::ConstPos const pos, Value **args,
                   Value &v) {
  /* ensure the argument is a function */
  state.forceValue(*args[0], pos);
  compat::forceLambda(state, *args[0], pos);

  printLambdaDocs(state, *args[0]);
  compat::mkNull(v);
}

/* Return position information of the given lambda. */
void prim_unsafeGetLambdaPos(EvalState &state, compat::ConstPos const pos,
                             Value **args, Value &v) {
  /* ensure the argument is a function */
  state.forceValue(*args[0], pos);
  compat::forceLambda(state, *args[0], pos);

  compat::mkPos(state, v, args[0]->lambda.fun->pos);
}

static RegisterPrimOp rp1 = compat::mkPrimop("__getDoc", {"func"}, "Get the textual docs for a function", prim_getDoc);
static RegisterPrimOp rp2 = compat::mkPrimop("__doc", {"func"}, "Print the docs for a function", prim_printDoc);
static RegisterPrimOp rp3 = compat::mkPrimop("__unsafeGetLambdaPos", {"func"}, "Get the position of some lambda", prim_unsafeGetLambdaPos);
