import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Dalang',
  description: 'Autonomous AI Cybersecurity Agent Framework',
  base: '/',
  head: [
    ['link', { rel: 'icon', href: '/dalang/logo.png' }],
  ],
  themeConfig: {
    logo: '/logo.png',
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Architecture', link: '/architecture/overview' },
      { text: 'Skills', link: '/skills/overview' },
      { text: 'GitHub', link: 'https://github.com/HasanH47/dalang' },
    ],
    sidebar: {
      '/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction', link: '/guide/getting-started' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Quick Start', link: '/guide/quick-start' },
            { text: 'Authentication', link: '/guide/authentication' },
          ],
        },
        {
          text: 'Usage',
          items: [
            { text: 'Scan Mode', link: '/guide/scan-mode' },
            { text: 'Auto-Pilot Mode', link: '/guide/auto-pilot' },
            { text: 'Interactive Mode', link: '/guide/interactive-mode' },
            { text: 'Web UI', link: '/guide/web-ui' },
          ],
        },
      ],
      '/architecture/': [
        {
          text: 'Architecture',
          items: [
            { text: 'Overview', link: '/architecture/overview' },
            { text: 'Core Engine', link: '/architecture/core-engine' },
            { text: 'LLM Providers', link: '/architecture/llm-providers' },
            { text: 'Executor & Security', link: '/architecture/executor' },
            { text: 'CDP Browser', link: '/architecture/cdp-browser' },
            { text: 'Web Server', link: '/architecture/web-server' },
          ],
        },
      ],
      '/skills/': [
        {
          text: 'Skill System',
          items: [
            { text: 'Overview', link: '/skills/overview' },
            { text: 'Creating Skills', link: '/skills/creating-skills' },
            { text: 'Built-in Skills', link: '/skills/built-in' },
            { text: 'Defensive Prompting', link: '/skills/defensive-prompting' },
          ],
        },
      ],
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/HasanH47/dalang' },
    ],
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026-present HasanH47',
    },
    search: {
      provider: 'local',
    },
  },
})
