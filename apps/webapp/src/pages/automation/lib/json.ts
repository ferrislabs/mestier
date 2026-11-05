export type JsonValue =
	| string
	| number
	| boolean
	| null
	| JsonValue[]
	| { [key: string]: JsonValue }

export type BodyValueKind = 'string' | 'number' | 'boolean' | 'json'
