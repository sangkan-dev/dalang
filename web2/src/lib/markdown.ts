import MarkdownIt from 'markdown-it';

const markdown = new MarkdownIt({
	html: false,
	linkify: true,
	breaks: true,
	typographer: true
});

const UNSAFE_PROTOCOL = /^(javascript:|vbscript:|data:)/i;

markdown.validateLink = (url: string): boolean => {
	const normalized = url.trim().replace(/\s/g, '');
	return !UNSAFE_PROTOCOL.test(normalized);
};

export function renderMarkdown(input: string): string {
	return markdown.render(input || '');
}

export function renderMarkdownRaw(input: string): string {
	return input || '';
}
