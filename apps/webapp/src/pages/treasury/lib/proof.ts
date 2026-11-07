export function basename(key: string): string {
	return key.split('/').at(-1) ?? key
}
