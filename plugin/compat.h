#include <eval.hh>

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

inline void mkString(nix::Value &v, const char *s) {
#ifdef NIX_2_6_0
  v.mkString(s);
#else
  nix::mkString(v, s);
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

inline void forceLambda(nix::EvalState &state, nix::Value &v, compat::ConstPos pos) {
#if defined(NIX_2_9_0)
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

} // namespace compat
