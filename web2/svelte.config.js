import adapter from '@sveltejs/adapter-static';

const buildTarget = process.env.DALANG_BUILD_TARGET === 'landing' ? 'landing' : 'dashboard';
const isLandingBuild = buildTarget === 'landing';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		prerender: {
			crawl: !isLandingBuild,
			handleUnseenRoutes: 'ignore',
			entries: isLandingBuild
				? ['/']
				: ['/dashboard', '/dashboard/chat', '/dashboard/reports', '/dashboard/skills', '/dashboard/settings']
		},
		adapter: adapter({
			pages: isLandingBuild ? 'build-landing' : 'build-dashboard',
			assets: isLandingBuild ? 'build-landing' : 'build-dashboard',
			fallback: isLandingBuild ? undefined : 'dashboard/index.html',
			strict: !isLandingBuild
		})
	}
};

export default config;
