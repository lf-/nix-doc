#include <cstring>
#include <eval.hh>
#include <nixexpr.hh>
#include <util.hh>
#include <variant>

// i don't even know why i need these now, but i will need em eventually!
namespace compat {
#ifdef NIX_2_9_0
using Pos = nix::PosIdx;
using ConstPos = Pos;
#else
using Pos = nix::Pos &;
using ConstPos = nix::Pos const &;
#endif

inline void mkNull(nix::Value &v) {
#ifdef NIX_2_6_0
  v.mkNull();
#else
  nix::mkNull(v);
#endif
}

inline void mkString(nix::Value &v, std::string_view s) {
#ifdef NIX_2_6_0
  v.mkString(s);
#else
  // We need to leak a string here since nix expects this to be owning. oopsie.
  nix::mkString(v, (new std::string{s})->c_str());
#endif
}

inline nix::Pos posForLambda(nix::EvalState &state, nix::ExprLambda &lam) {
#ifdef NIX_2_9_0
  return state.positions[lam.pos];
#else
  (void)state;
  return lam.pos;
#endif
}

inline void mkPos(nix::EvalState &state, nix::Value &v, compat::Pos pos) {
#if defined(NIX_2_9_0)
  state.mkPos(v, pos);
#elif defined(NIX_2_4_0)
  state.mkPos(v, nix::ptr{&pos});
#else
  state.mkPos(v, &pos);
#endif
}

inline void forceLambda(nix::EvalState &state, nix::Value &v,
                        compat::ConstPos pos) {
  // author's note: lol, lmao
  // bonus author's note: the above was written before 2.13.1 reverted the
  // error builder. lol lmao.
#if defined(NIX_2_13_0) && !defined(NIX_2_13_1)
  if (!v.isLambda()) {
    state.error("value is %1% while a lambda was expected", nix::showType(v))
        .withTrace(pos, "")
        .debugThrow<nix::TypeError>();
  }
#elif defined(NIX_2_9_0)
  if (!v.isLambda()) {
    state.throwTypeError(pos, "%2%: value is %1% while a lambda was expected",
                         v);
  }
#elif defined(NIX_2_4_0)
  (void)state;
  if (!v.isLambda()) {
    throwTypeError(pos, "%2%: value is %1% while a lambda was expected", v);
  }
#else
  (void)state;
  if (v.type != tLambda) {
    throwTypeError("%2%: value is %1% while a lambda was expected", v, pos);
  }
#endif
}

inline std::string fileForPos(nix::Pos const &pos) {
#if defined(NIX_2_13_0)
  return std::visit(
      nix::overloaded{[](nix::Pos::none_tag) { return std::string{""}; },
                      [](nix::Pos::Stdin) { return std::string{""}; },
                      [](nix::Pos::String) { return std::string{""}; },
                      [](nix::Path p) { return std::string{p}; }},
      pos.origin);
#else
  return pos.file;
#endif
}

} // namespace compat
