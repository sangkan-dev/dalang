import MarkdownIt from 'markdown-it';

const markdown = new MarkdownIt({
	html: false,
	linkify: true,
	breaks: true,
	typographer: true
});

export function renderMarkdown(input: string): string {
	return markdown.render(input || '');
}
