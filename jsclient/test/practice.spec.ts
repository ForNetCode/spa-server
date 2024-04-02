//@ts-ignore
import parse from '@pushcorn/hocon-parser'
import walkdir from "walkdir";
test("hocon-parser can load files ", async () => {
    const data = await parse({url: './test/example.conf'})
    expect(data).toStrictEqual({a:{b:2}})
})


test("read directory", async () => {

    const files = await walkdir.async('./test', {return_object: true})
    Object.keys(files).forEach((path) => {
        const stat = files[path]
        if(stat.isFile()) {
            console.log(path)
        }
    })
})
