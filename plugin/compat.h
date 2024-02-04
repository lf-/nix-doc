#include <cstring>
#include <eval.hh>
#include <nixexpr.hh>
#include <primops.hh>
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
  //
  // it appears that it is back in 2.14.0.
#if defined(NIX_2_14_0) || (defined(NIX_2_13_0) && !defined(NIX_2_13_1))
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

#if defined(NIX_2_16_0)
using SourcePathT = nix::SourcePath;
inline auto sourcePathToString(SourcePathT p) -> std::string { return p.to_string(); };
#else
using SourcePathT = nix::Path;
inline auto sourcePathToString(SourcePathT p) -> std::string { return std::string{p}; };
#endif

#if defined(NIX_2_20_0)
using EmptyPos = std::monostate;
#elif defined(NIX_2_13_0)
using EmptyPos = nix::Pos::none_tag;
#else
#endif

inline std::string fileForPos(nix::Pos const &pos) {
#if defined(NIX_2_13_0)
  return std::visit(
      nix::overloaded{[](EmptyPos) { return std::string{""}; },
                      [](nix::Pos::Stdin) { return std::string{""}; },
                      [](nix::Pos::String) { return std::string{""}; },
                      [](SourcePathT p) { return sourcePathToString(p); }},
      pos.origin);
#else
  return pos.file;
#endif
}

inline nix::RegisterPrimOp
mkPrimop(const std::string name, std::vector<std::string> args,
         const char *docs,
         void (*primop)(nix::EvalState &, ConstPos const pos, nix::Value **args,
                        nix::Value &v)) {
#if defined(NIX_2_17_0)
  return nix::RegisterPrimOp((nix::PrimOp){
      .name = name,
      .args = args,
      .arity = args.size(),
      .doc = docs,
      .fun = primop,
      .experimentalFeature = {},
  });
#else
  (void)docs;
  return nix::RegisterPrimOp{name, args.size(), primop};
#endif
}

} // namespace compat
