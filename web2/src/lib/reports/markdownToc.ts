export type ReportTocItem = {
	level: 2 | 3;
	text: string;
	id: string;
};

function slugify(text: string): string {
	const base = text
		.normalize('NFKD')
		.replaceAll(/[\u0300-\u036f]/g, '')
		.toLowerCase()
		.replaceAll(/[^a-z0-9\s-]/g, '')
		.trim()
		.replaceAll(/\s+/g, '-')
		.slice(0, 80);
	return base || 'bagian';
}

/** Judul ## / ### dari markdown, urutan sama dengan heading yang dirender. */
export function extractReportToc(markdown: string): ReportTocItem[] {
	const items: ReportTocItem[] = [];
	const slugUses = new Map<string, number>();
	for (const line of markdown.split('\n')) {
		const m = /^(#{2,3})\s+(.+?)\s*$/.exec(line.trimEnd());
		if (!m) continue;
		const level = m[1].length as 2 | 3;
		if (level !== 2 && level !== 3) continue;
		let text = m[2].replace(/\s+#+\s*$/, '').trim();
		const base = slugify(text);
		const n = (slugUses.get(base) ?? 0) + 1;
		slugUses.set(base, n);
		const id = n === 1 ? base : `${base}-${n}`;
		items.push({ level, text, id });
	}
	return items;
}
