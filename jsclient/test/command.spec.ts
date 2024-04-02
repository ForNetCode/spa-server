
import execa from 'execa';
import path from 'path'

async function runCommand(param: string[], options: execa.Options<string> = {}) {
    const result = await execa('npx', ['ts-node','./test/command_test_run.ts', ...param], options)
    return result.stdout
}
test('info', async () => {
    expect(await runCommand(['info'])).toBe('[]')
})

test('upload', async () => {
    const exampleProjectDir = path.resolve(__dirname, '../../example/js-app-example')
    try {
        const result = await runCommand(['upload', path.join(exampleProjectDir, 'build'), 'self.noti.link'])
        console.log(result)
    }catch(e) {
        // expect error
    }
})
test('release', async () => {
    try {
        const result = await runCommand(['release', 'self.noti.link'])
        console.log(result)
    }catch(e){

    }
})




