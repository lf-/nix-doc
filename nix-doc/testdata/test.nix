{parm1, parm2}:
let n = import <nixpkgs> {};
    ff = 2;
in
{
  /* This is a function blah blah
     with a long ass doc comment
   */
   the-fn = a: b: {z = a; y = b;};

   # this one
   # has multiple
   # comments
   the-snd-fn = {b, /* doc */ c}: {};

   # sorry...
   a.b.c = a: 1;

   inherit (n) grub hello;
   inherit ff;
 }
