export class ExternalObject<T> {
    readonly '': {
        readonly '': unique symbol
        [K: symbol]: T
    }
}
export function run(): void