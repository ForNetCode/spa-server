import axios, {AxiosInstance, AxiosResponse} from "axios";
import {fileFromPath} from "formdata-node/file-from-path";

import fs from "fs";
import crypto from "crypto";
import chalk from "chalk";
import walkdir from "walkdir";
import * as path from "path";
import {asyncFilter, asyncFindIndex} from 'modern-async'


export interface SPAClientConfig {
    address: string
    authToken: string
}

export enum UploadingStatus {
    Uploading = 0,
    Finish = 1,
}

export enum GetDomainPositionStatus {
    NewDomain = 0,
    NewVersion = 1,
    Uploading = 2
}

export interface UploadDomainPositionResp {
    path: string
    version: number
    status: GetDomainPositionStatus
}

export interface CertInfoResp {
    begin: string,
    end: string,
    host: string,
}

export interface ShortMetaData {
    path: string
    md5: string
    length: number
}

export interface DomainInfo {
    domain: string
    current_version: number
    versions: number[]
}

export default class SPAClient {
    // private config:SPAClientConfig
    private http: AxiosInstance

    constructor(config: SPAClientConfig) {
        // this.config = config
        this.http = axios.create({
            baseURL: config.address,
            headers: {
                "Authorization": `Bearer ${config.authToken}`
            }
        })
        this.http.interceptors.response.use((resp) => resp, (error) => {
            return Promise.reject(error.cause)
        })
    }

    static init(config: SPAClientConfig) {
        return new SPAClient(config)
    }

    public getDomainInfo(domain?: string) {
        return this.http.get('/status', {params: {domain}}).then(resp<DomainInfo[]>)
    }

    public changeUploadingStatus(domain: string, version: number, status: UploadingStatus) {
        return this.http.post('/files/upload_status', {
            domain,
            version,
            status
        }).then(emptyResp)
    }

    public releaseDomainVersion(domain: string, version?: number) {
        return this.http.post('/update_version', {
            domain,
            version
        }).then(resp<string>)
    }

    public reloadSPAServer() {
        return this.http.post('/reload').then(emptyResp)
    }


    public removeFiles(domain?: string, maxReserve?: number) {
        return this.http.post('/files/delete', {domain, max_reserve: maxReserve}).then(emptyResp)
    }

    public async uploadFile(domain: string, version: number, key: string, path: string) {
        if (key.indexOf('\\') !== -1) {
            throw 'key should be unix like, not windows'
        }
        //const fileStream = fs.createReadStream(path)
        const form = new FormData()
        // form.append('domain', domain)
        // form.append('version', version.toString())
        // form.append('path', key)
        //@ts-ignore
        form.append('file', await fileFromPath(path))
        return this.http.post('/file/upload', form, {
            params: {domain, version, path: key}
        }).then((resp) => {
            if (resp.status !== 200) {
                throw resp.data
            }
        })
    }

    public async uploadFilesParallel(domain: string, version: number | undefined, filePath: string, parallel: number = 3) {
        const absolutePath = path.resolve(filePath)
        if (!(fs.existsSync(absolutePath) && fs.statSync(absolutePath).isDirectory())) {
            throw `path:${path} is not directory or does not exists`
        }
        const files = await walkdir.async(absolutePath, {return_object: true})
        if (Object.keys(files).length == 0) {
            throw `path:${path} has no files`
        }
        let realVersion: number;
        if (!version) {
            const positionResp = await this.getUploadPosition(domain)
            if (positionResp.status === GetDomainPositionStatus.NewDomain) {
                console.log(chalk.green(`domain:${domain} is new in server!`))
            }
            realVersion = positionResp.version
        } else {
            realVersion = version
        }
        console.log(chalk.green("begin to fetch server file metadata with md5, you may need to wait if there are large number of files."))
        const serverMetaData = await this.getFileMetadata(domain, realVersion)
        if (!serverMetaData.length) {
            console.log(chalk.green(`There are ${serverMetaData.length} files already in server`))
        }
        const serverMetaDataMap = serverMetaData.reduce((result, item) => {
            result[item.path] = item
            return result
        }, {} as { [key: string]: ShortMetaData })
        const uploadingFiles = Object.keys(files).reduce((result, filePath) => {
            const fileStat = files[filePath]
            if (fileStat.isFile()) {
                //TODO: check if key is correct
                let key = filePath.replace(absolutePath, '').replace(/\\/g, '/')
                const meta = serverMetaDataMap[key]
                if (!(meta && meta.length == fileStat.size && meta.md5 === crypto.createHash("md5").update(fs.readFileSync(filePath)).digest('hex'))) {
                    result.push({key, filePath})
                }
            }
            return result

        }, [] as { key: string, filePath: string }[])
        if (!uploadingFiles.length) {
            console.log(chalk.green("all files already upload"))
            await this.changeUploadingStatus(domain, realVersion, UploadingStatus.Finish)
            return
        }
        console.log(chalk.green(`there are ${uploadingFiles.length} files to upload`))
        await this.changeUploadingStatus(domain, realVersion, UploadingStatus.Uploading)
        console.log(chalk.green(`prepare files to upload and change:${domain}:${realVersion} status: Uploading`))
        const failUploadFiles = await asyncFilter(uploadingFiles, async ({key, filePath}) => {
            const uploadResult = await this.retryUpload(domain, realVersion, key, filePath)
            if (uploadResult === -1) {
                console.error(`[Fail]${key}`)
                return true
            } else {
                console.info(chalk.green(`[Success]${key}`))
                return false
            }
        }, parallel)
        if (failUploadFiles.length) {
            throw `there are ${failUploadFiles.length} file(s) uploaded fail`
        } else {
            await this.changeUploadingStatus(domain, realVersion, UploadingStatus.Finish)
        }

    }

    retryUpload(domain: string, version: number, key: string, path: string, count: number = 3) {
        return asyncFindIndex(Array(count), async (_) => {
            try {
                await this.uploadFile(domain, version, key, path)
            } catch (e) {
                console.error(e)
                return false
            }
            return true
        }, 1, true)
    }

    public getFileMetadata(domain: string, version: number) {
        return this.http.get('/files/metadata', {
            params: {domain, version}
        }).then(resp<ShortMetaData[]>)
    }

    public getUploadPosition(domain: string) {
        return this.http.get('/upload/position', {
            params: {domain, format: 'Json'}
        }).then(resp<UploadDomainPositionResp>)
    }
    public revokeVersion(domain:String, version: number) {
        return this.http.post(`/files/revoke_version`, {domain, version}).then(emptyResp)
    }
    public getCertInfo(domain?:string) {
        return this.http.get('/cert/acme', {params: {domain}}).then(resp<CertInfoResp[]>)
    }
}


function resp<T>(resp: AxiosResponse) {
    if (resp.status === 200) {
        return resp.data as T
    } else {
        throw resp.data as string
    }
}

function emptyResp(resp: AxiosResponse) {
    if (resp.status !== 200) {
        throw resp.data as string
    }
}
