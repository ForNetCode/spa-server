import {defineConfig} from "vitepress";

export default defineConfig({
    lang: 'en-US',
    title: 'SPA-SERVER',
    description: 'deploy SPA so easy!',
    base: '/spa-server/',
    themeConfig: {
        repo: 'timzaak/spa-server',
        docsDir: 'docs',
        docsBranch: 'master',
        editLinks: true,
        editLinkText: 'Edit this page on GitHub',
        lastUpdated: 'Last Updated',
        nav: [
            {text: 'Guide', link: '/', activeMatch: '^/$|^/guide/'},
            {
                text: 'Dev Log',
                link: '/develop/change-log',
                activeMatch: '^/develop/'
            },
            {
                text: 'Release Notes',
                link: 'https://github.com/timzaak/spa-server/releases'
            }
        ],
        sidebar: {
            '/guide/': getGuideSidebar(),
            '/develop/': getDevelopSideBar(),
            '/': getGuideSidebar(),
        }
    },
})

function getGuideSidebar() {
    return [
        {
            text: 'Introduction',
            children: [
                {text: 'What is SPA-Server?', link: '/'},
                {text: 'Getting Started', link: '/guide/getting-started'},
                {text: 'Break Changes', link: '/guide/break-changes'}
            ],
        },
        {
            text: 'SPA-Server', children: [
                {text: 'Configuration', link: '/guide/spa-server-configuration'},
                {text: 'Http API', link: '/guide/spa-server-api'},
                {text: 'Package', link: '/guide/spa-server-release-pacakge'},
            ]
        },
        {
            text: 'SPA-Client',
            children: [
                {text: 'Configuration', link: '/guide/spa-client-configuration'},
                {text: 'Command Line', link: '/guide/spa-client-command-line'},
                {text: 'NPM Package', link: '/guide/spa-client-npm-package'}
            ]
        }, {
            text: "Advanced",
            children: [
                {text: 'Uploading File Process', link: '/guide/uploading-file-process'}
            ]
        }
    ]
}

function getDevelopSideBar() {
    return [
        {
            text: 'Develop Log',
            children: [
                {text: 'Change Log', link: '/develop/change-log'},
                {text: 'Develop Tip', link: '/develop/develop-tips'},
                {text: 'Roadmap', link: '/develop/roadmap'},

            ]
        }]
}