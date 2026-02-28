/**
 * Markdown renderer using marked + highlight.js
 */
import { marked } from 'marked';
import hljs from 'highlight.js/lib/core';
import type { LanguageFn } from 'highlight.js';
import javascript from 'highlight.js/lib/languages/javascript';
import bash from 'highlight.js/lib/languages/bash';
import json from 'highlight.js/lib/languages/json';
import python from 'highlight.js/lib/languages/python';
import xml from 'highlight.js/lib/languages/xml';
import sql from 'highlight.js/lib/languages/sql';
import yaml from 'highlight.js/lib/languages/yaml';

// Register common languages
const languages: Record<string, LanguageFn> = {
  javascript,
  bash,
  shell: bash,
  json,
  python,
  xml,
  html: xml,
  sql,
  yaml,
};

for (const [name, lang] of Object.entries(languages)) {
  hljs.registerLanguage(name, lang);
}

marked.setOptions({
  breaks: true,
  gfm: true,
});

const renderer = new marked.Renderer();
renderer.code = function ({ text, lang }: { text: string; lang?: string }): string {
  const language = lang && hljs.getLanguage(lang) ? lang : undefined;
  const highlighted = language
    ? hljs.highlight(text, { language }).value
    : hljs.highlightAuto(text).value;
  return `<pre><code class="hljs${language ? ` language-${language}` : ''}">${highlighted}</code></pre>`;
};
marked.use({ renderer });

export function renderMarkdown(text: string): string {
  if (!text) return '';
  return marked.parse(text) as string;
}
