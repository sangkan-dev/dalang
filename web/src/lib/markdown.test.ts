import { describe, it, expect } from 'vitest';
import { renderMarkdown } from './markdown.ts';

describe('renderMarkdown', () => {
  it('should return empty string for falsy input', () => {
    expect(renderMarkdown('')).toBe('');
  });

  it('should render basic paragraph', () => {
    const result = renderMarkdown('Hello world');
    expect(result).toContain('<p>Hello world</p>');
  });

  it('should render bold text', () => {
    const result = renderMarkdown('**bold**');
    expect(result).toContain('<strong>bold</strong>');
  });

  it('should render code blocks with hljs classes', () => {
    const result = renderMarkdown('```bash\necho hello\n```');
    expect(result).toContain('<pre>');
    expect(result).toContain('<code');
    expect(result).toContain('hljs');
  });

  it('should render inline code', () => {
    const result = renderMarkdown('use `nmap` tool');
    expect(result).toContain('<code>nmap</code>');
  });

  it('should render links', () => {
    const result = renderMarkdown('[test](http://example.com)');
    expect(result).toContain('href="http://example.com"');
  });

  it('should render unordered lists', () => {
    const result = renderMarkdown('- item 1\n- item 2');
    expect(result).toContain('<li>item 1</li>');
    expect(result).toContain('<li>item 2</li>');
  });
});
