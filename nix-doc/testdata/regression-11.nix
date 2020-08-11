{}: {
  /* Create an --{with,without}-<feat> string that can be passed to
     standard GNU Autoconf scripts.

     Example:
       withFeature true "shared"
       => "--with-shared"
       withFeature false "shared"
       => "--without-shared"
  */
  withFeature = with_: feat: "--${if with_ then "with" else "without"}-${feat}";

  /* Create an --{with-<feat>=<value>,without-<feat>} string that can be passed to
     standard GNU Autoconf scripts.

     Example:
       with_Feature true "shared" "foo"
       => "--with-shared=foo"
       with_Feature false "shared" (throw "ignored")
       => "--without-shared"
  */
  withFeatureAs = with_: feat: value: withFeature with_ feat + optionalString with_ "=${value}";

  /* Create a fixed width string with additional prefix to match
     required width.

     This function will fail if the input string is longer than the
     requested length.

     Type: fixedWidthString :: int -> string -> string

     Example:
       fixedWidthString 5 "0" (toString 15)
       => "00015"
  */
  fixedWidthString = width: filler: str:
    let
      strw = lib.stringLength str;
      reqWidth = width - (lib.stringLength filler);
    in
      assert lib.assertMsg (strw <= width)
        "fixedWidthString: requested string length (${
          toString width}) must not be shorter than actual length (${
            toString strw})";
      if strw == width then str else filler + fixedWidthString reqWidth filler str;
}
