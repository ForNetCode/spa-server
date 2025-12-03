import execa from 'execa';
import path from 'path'

const LOCAL_HOST = 'local.fornetcode.com';

async function runCommand(param: string[], options: execa.Options<string> = {}) {
    const result = await execa('npx', ['ts-node', './test/command_test_run.ts', ...param], options)
    return result.stdout
}

test('info', async () => {
    expect(await runCommand(['info'])).toBe('[]')
})

test('upload', async () => {
    const exampleProjectDir = path.join(path.resolve(__dirname, '../../example/js-app-example'), 'build')
    console.log(`upload path ${exampleProjectDir}`)
    const result = await runCommand(['upload', exampleProjectDir, LOCAL_HOST])
    console.log(result)

})
/*
test('release', async () => {
    const result = await runCommand(['release', LOCAL_HOST])
    console.log(result)
    // expect(await runCommand(['info'])).toBe(`[{"domain":"${LOCAL_HOST}","current_version":1,"versions":[1]}]`)
})
*/
