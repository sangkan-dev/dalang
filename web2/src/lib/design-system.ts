export const colorTokens = {
	andesite: '#0d0d0d',
	andesiteLight: '#1a1a1a',
	gold: '#cba153',
	goldBright: '#e4be6a',
	baseText: '#e0e0e0',
	ash: '#a0a0a0',
	smoke: '#666666',
	rust: '#dd6b20'
} as const;

export const spacingTokens = {
	xxs: '0.25rem',
	xs: '0.5rem',
	sm: '0.75rem',
	md: '1rem',
	lg: '1.5rem',
	xl: '2rem',
	xxl: '3rem',
	xxxl: '4rem'
} as const;

export const radiusTokens = {
	sm: '0.5rem',
	md: '0.875rem',
	lg: '1.25rem',
	pill: '999px'
} as const;

export const shadowTokens = {
	hairline: '0 0 0 1px rgb(255 255 255 / 8%)',
	goldSoft: '0 0 36px rgb(203 161 83 / 18%)',
	elevated: '0 14px 34px rgb(0 0 0 / 35%)',
	panel: '0 18px 50px rgb(0 0 0 / 40%)'
} as const;

export const typographyTokens = {
	ui: "'Plus Jakarta Sans', 'Inter', 'Segoe UI', 'Noto Sans', system-ui, sans-serif",
	mono: "'JetBrains Mono', 'Cascadia Mono', monospace",
	javanese: "'Noto Sans Javanese', 'Noto Sans Javanese UI', 'Noto Sans', 'Segoe UI', sans-serif"
} as const;

export const a11yTokens = {
	focusRingWidth: '2px',
	focusRingOffset: '2px',
	focusRingColor: colorTokens.goldBright,
	selectionBackground: 'rgb(228 190 106 / 28%)',
	selectionText: '#fff8eb'
} as const;

export const motionTokens = {
	quick: '150ms',
	base: '250ms',
	slow: '400ms'
} as const;

/** Max width for dashboard shell (keep in sync with `layout.css` `.dashboard-shell`) */
export const dashboardContentMaxWidth = 'min(1280px, calc(100% - 2 * var(--space-lg)))';

export type Platform = 'desktop' | 'mobile';

export function getUiFallbackChain(platform: Platform): readonly string[] {
	if (platform === 'mobile') {
		return ['Plus Jakarta Sans', 'Noto Sans', 'Roboto', 'system-ui'];
	}

	return ['Plus Jakarta Sans', 'Inter', 'Segoe UI', 'Noto Sans', 'system-ui'];
}

export function getJavaneseFallbackChain(platform: Platform): readonly string[] {
	if (platform === 'mobile') {
		return ['Noto Sans Javanese', 'Noto Sans', 'Roboto', 'sans-serif'];
	}

	return ['Noto Sans Javanese', 'Noto Sans Javanese UI', 'Noto Sans', 'Segoe UI', 'sans-serif'];
}
