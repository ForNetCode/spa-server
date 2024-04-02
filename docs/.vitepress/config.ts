import { defineConfig} from "vitepress";

export default defineConfig({
    lang: 'en-US',
    title: 'SPA-SERVER',
    description: 'deploy SPA so easy!',
    base: '/spa-server/',
    themeConfig: {
        lastUpdated: {
            text: 'Last Updated'
        },
        search : {
            provider: 'algolia',
            options: {
                appId: 'NBNHWJCAL4',
                apiKey: 'fa9bd1600dd455c6fc927a6fbafcd7b5',
                indexName: 'spa-server',
            }
        },
        nav: [
            {text: 'Guide', link: '/', activeMatch: '^/$|^/guide/'},
            {
                text: 'Dev Log',
                link: '/develop/change-log',
                activeMatch: '^/develop/'
            },
            {
                text: 'Release Notes',
                link: 'https://github.com/fornetcode/spa-server/releases'
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
            items: [
                {text: 'What is SPA-Server?', link: '/'},
                {text: 'Getting Started', link: '/guide/getting-started'},
                {text: 'Break Changes', link: '/guide/break-changes'}
            ],
        },
        {
            text: 'SPA-Server',
            items: [
                {text: 'Configuration', link: '/guide/spa-server-configuration'},
                {text: 'Http API', link: '/guide/spa-server-api'},
                {text: 'Package', link: '/guide/spa-server-release-package'},
            ]
        },
        {
            text: 'SPA-Client',
            items: [
                {text: 'NPM Package', link: '/guide/spa-client-npm-package'},
                {text: 'Command Line', link: '/guide/spa-client-command-line'},
            ]
        },
    ]
}

function getDevelopSideBar(){
    return [
        {
            text: 'Develop Log',
            items: [
                {text: 'Change Log', link: '/develop/change-log'},
                {text: 'Develop Tip', link: '/develop/develop-tips'},
                {text: 'Roadmap', link: '/develop/roadmap'},

            ]
        }]
}