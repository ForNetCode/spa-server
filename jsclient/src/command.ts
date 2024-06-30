import SPAClient from "./index";
//@ts-ignore
import parse from '@pushcorn/hocon-parser'
import {
    binary,
    command,
    extendType,
    number,
    option,
    optional,
    positional,
    run,
    string,
    subcommands,
    Type
} from "cmd-ts";
import {ExistingPath} from 'cmd-ts/batteries/fs';
import {version as Version} from '../package.json'
import chalk from "chalk";

export const Integer: Type<string, number> = extendType(number, {
    async from(n) {
        if (Math.round(n) !== n) {
            throw new Error('This is a floating-point number');
        }
        return n;
    },
});


const configDirOption = option({
    type: optional(ExistingPath), short: 'c', long: 'config-dir', description: 'config dir',
    defaultValue() {
        return process.env.SPA_CLIENT_CONFIG
    }
})
const domain = positional({type: string, displayName: 'domain'})

const version = positional({type: Integer, displayName: 'version'});
const versionOptional = positional({type: optional(Integer), displayName: 'version'});

async function getClient(configPath: string | undefined) {
    //load config
    const hocon = configPath ? await parse({url: configPath}) : {}
    // address: string
    // authToken: string
    const address: string | undefined = hocon?.server?.address ?? process.env.SPA_SERVER_ADDRESS
    const authToken: string | undefined = hocon?.server?.auth_token ?? process.env.SPA_SERVER_AUTH_TOKEN

    if (!address) {
        return Promise.reject("server.address could not get")
    }
    if (!authToken) {
        return Promise.reject("server.auth_token could not get")
    }
    return new SPAClient({address, authToken})
}

function writeResult(func: () => Promise<any>) {
    func().then((v) => console.log(chalk.green(v))).catch((e) => {
        console.error(e)
        process.exit(-1)
    })
}

const info = command({
    name: 'info',
    args: {
        domain: positional({type: optional(string), displayName: 'domain'}),
        config: configDirOption,
    },
    handler: ({domain, config}) => {
        writeResult(async () => {
            const client = await getClient(config)
            return await client.getDomainInfo(domain)
        })
    }
})

const upload = command({
    name: 'upload',
    args: {
        path: positional({type: ExistingPath, displayName: 'path'}),
        domain,
        version: versionOptional,
        parallel: option({type: optional(Integer), short: 'p', long: 'parallel'}),
        config: configDirOption,
    },
    handler({path, domain, version, parallel, config}) {
        writeResult(async () => {
            const client = await getClient(config)
            const parallelParam = parallel ?? (process.env.SPA_UPLOAD_PARALLEL ? Number(process.env.SPA_UPLOAD_PARALLEL) : undefined) ?? 3
            await client.uploadFilesParallel(domain, version, path, parallelParam)
            return "upload files finish"
        })
    }
})
const release = command({
    name: 'release',
    args: {
        domain,
        version: versionOptional,
        config: configDirOption,
    },
    handler({domain, version, config}) {
        writeResult(async () => {
            const client = await getClient(config)
            return await client.releaseDomainVersion(domain, version)
        })
    }
})

const reload = command({
    name: 'reload',
    args: {config: configDirOption,},
    handler({config}) {
        writeResult(async () => {
            const client = await getClient(config)
            await client.reloadSPAServer()
            return "reload successful"
        })
    }
})
const deleteCmd = command({
    name: 'delete',
    args: {
        domain: positional({type: optional(string), displayName: 'domain'}),
        maxReserve: positional({type: optional(Integer), displayName: 'maxReserve'}),
        config: configDirOption,
    },
    handler({domain, maxReserve, config}) {
        writeResult(async () => {
            const client = await getClient(config)
            await client.removeFiles(domain, maxReserve)
            return "delete successful"
        })
    }
})
const revokeVersionCmd = command({
    name: 'revoke',
    args: {
        domain,
        version,
        config: configDirOption
    },
    handler({domain, version, config}) {
        writeResult(async () => {
            const client = await getClient(config)
            await client.revokeVersion(domain, version)
            return "revoke successful"
        })
    }
})


export const cmd = subcommands({
    name: 'spa-client',
    description: 'js command line for spa-server',
    version: Version,
    cmds: {info, upload, release, reload, delete: deleteCmd, revoke: revokeVersionCmd}
})
export default function runCommand() {
    run(binary(cmd), process.argv)
}
