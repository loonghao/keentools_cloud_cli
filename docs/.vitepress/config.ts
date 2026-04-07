import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'keentools-cloud',
  description: 'Unofficial CLI for the KeenTools Cloud 3D Head Reconstruction API',
  base: '/keentools_cloud_cli/',

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Commands', link: '/commands/' },
      { text: 'Agent Integration', link: '/agent/' },
      {
        text: 'GitHub',
        link: 'https://github.com/loonghao/keentools_cloud_cli',
      },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction', link: '/guide/' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Configuration', link: '/guide/configuration' },
            { text: 'Quick Start', link: '/guide/getting-started' },
          ],
        },
      ],
      '/commands/': [
        {
          text: 'Commands',
          items: [
            { text: 'Overview', link: '/commands/' },
            { text: 'auth', link: '/commands/auth' },
            { text: 'init', link: '/commands/init' },
            { text: 'upload', link: '/commands/upload' },
            { text: 'process', link: '/commands/process' },
            { text: 'status', link: '/commands/status' },
            { text: 'download', link: '/commands/download' },
            { text: 'info', link: '/commands/info' },
            { text: 'run', link: '/commands/run' },
            { text: 'ephemeral', link: '/commands/ephemeral' },
            { text: 'schema', link: '/commands/schema' },
            { text: 'self-update', link: '/commands/self-update' },
          ],
        },
      ],
      '/agent/': [
        {
          text: 'Agent Integration',
          items: [
            { text: 'Overview', link: '/agent/' },
            { text: 'JSON Output', link: '/agent/json-output' },
            { text: 'Pipelines', link: '/agent/pipelines' },
            { text: 'Schema Introspection', link: '/agent/schema' },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/loonghao/keentools_cloud_cli' },
    ],

    footer: {
      message:
        'This is an unofficial project and is not affiliated with or endorsed by KeenTools.',
      copyright: 'Released under the MIT License.',
    },

    search: {
      provider: 'local',
    },
  },
})
