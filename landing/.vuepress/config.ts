import { defineUserConfig } from 'vuepress'
import { viteBundler } from '@vuepress/bundler-vite'
import { defaultTheme } from '@vuepress/theme-default'

export default defineUserConfig({
  bundler: viteBundler(),
  lang: 'en-US',
  title: 'NeonD',
  description: 'DX-focused control plane for PostgreSQL',
  head: [['link', { rel: 'icon', href: '/favicon.svg' }]],
  theme: defaultTheme({
    logo: '/logo.svg',
    repo: 'matisiekpl/neond',
    docsRepo: 'matisiekpl/neond',
    docsDir: 'landing',
    navbar: [
      { text: 'Docs', link: '/docs/' },
    ],
    sidebar: {
      '/docs/': [
        {
          text: 'Getting started',
          collapsible: false,
          children: [
            '/docs/README.md',
            '/docs/installation.md',
            '/docs/quickstart.md',
            '/docs/configuration.md',
          ],
        },
        {
          text: 'Using neond',
          collapsible: false,
          children: [
            '/docs/branching.md',
            '/docs/import.md',
            '/docs/pgbouncer.md',
            '/docs/tls-sni.md',
          ],
        },
        {
          text: 'Operating neond',
          collapsible: false,
          children: [
            '/docs/startup.md',
            '/docs/storage.md',
            '/docs/backups.md',
          ],
        },
      ],
    },
  }),
})
