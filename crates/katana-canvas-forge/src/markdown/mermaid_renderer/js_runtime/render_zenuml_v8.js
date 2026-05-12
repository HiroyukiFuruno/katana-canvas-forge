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
  var svg = zenuml.renderToSvg(stripped).svg;
  if (__zenuml_dark__) {
    // renderToSvg ignores the options argument in @zenuml/core 3.47.9 (GZ(t,e)
    // never reads `e`). Apply dark theme by injecting CSS variable overrides
    // directly into the SVG so that var(--color-*) references resolve correctly.
    var darkStyle = '<style>:root,svg{' +
      '--color-bg-base:#111628;' +
      '--color-text-base:#cecfd2;' +
      '--color-border-base:#cecfd2;' +
      '--color-border-frame:#cecfd2;' +
      '--color-text-fragment-header:#cecfd2;' +
      '--color-bg-fragment-header:#5964f2;' +
      '--color-text-fragment:#cecfd2;' +
      '--color-message-arrow:#536fff;' +
      '--color-text-participant:#536fff;' +
      '--color-bg-participant:#5964f2;' +
      '--color-border-participant:#cecfd2;' +
      '--color-bg-occurrence:#5964f2;' +
      '--color-border-occurrence:#cecfd2;' +
    '}</style>';
    svg = svg.replace(/(<svg\b[^>]*>)/, '$1' + darkStyle);
  }
  return svg;
})();
