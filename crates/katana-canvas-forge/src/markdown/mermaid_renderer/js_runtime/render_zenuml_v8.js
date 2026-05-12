// Bridge script for @zenuml/core V8 rendering.
// Input:  __zenuml_source__ (global var set by preamble script)
//         __zenuml_dark__   (boolean, set by preamble script)
// Output: SVG string (last expression, captured by DiagramV8Runtime)
(function () {
  var source = __zenuml_source__;
  // Strip the leading "zenuml" keyword line.
  // Mermaid fenced code blocks include it as the diagram-type token,
  // but @zenuml/core's renderToSvg() treats it as a participant name.
  var stripped = source.replace(/^zenuml[^\n]*\n?/, "");
  var options = __zenuml_dark__ ? { theme: "dark" } : {};
  return zenuml.renderToSvg(stripped, options).svg;
})();
