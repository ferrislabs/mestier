/**
 * Pure helpers for the `{{ connectors.<id>.output.<path> }}` expression
 * syntax (see `libs/core/src/domain/automation/expression/parser.rs`) — no
 * React, no fetch. `ExpressionPicker` (`ui/expression-picker.tsx`) is the
 * only caller.
 *
 * Nothing here evaluates or validates an expression: that stays the
 * backend's job, enforced at save time (`GraphError`) and at run time. This
 * only turns a connector's `output_example` into candidate paths, and a
 * chosen path into the exact string to insert.
 */

const MAX_DEPTH = 4
const MAX_PATHS = 50
const MAX_ARRAY_SAMPLE = 1

export interface ExamplePath {
	/** `''` means "the whole output" — never has a leading `.`. */
	path: string
	/** Short, one-line rendering of the value at that path, for the picker's
	 * list — never the full value: a large nested object would swamp it. */
	preview: string
}

/** Walks a connector's `output_example` into the paths a user can pick from.
 * Stops descending past `MAX_DEPTH` and caps the total list at `MAX_PATHS` —
 * an example payload is illustrative, not exhaustively enumerable. Only the
 * first array element is sampled, since an expression path is static
 * (`lines[1].total`, never `lines[i].total`). */
export function flattenExamplePaths(value: unknown): ExamplePath[] {
	const paths: ExamplePath[] = []
	visit(value, '', 0, paths)
	return paths.slice(0, MAX_PATHS)
}

function visit(
	value: unknown,
	path: string,
	depth: number,
	out: ExamplePath[],
): void {
	if (out.length >= MAX_PATHS) return

	out.push({ path, preview: previewOf(value) })

	if (depth >= MAX_DEPTH) return

	if (Array.isArray(value)) {
		value.slice(0, MAX_ARRAY_SAMPLE).forEach((item, index) => {
			visit(item, `${path}[${index}]`, depth + 1, out)
		})
		return
	}

	if (value !== null && typeof value === 'object') {
		for (const [key, child] of Object.entries(value)) {
			visit(child, path === '' ? key : `${path}.${key}`, depth + 1, out)
		}
	}
}

function previewOf(value: unknown): string {
	if (value === null) return 'null'
	if (typeof value === 'string') return `"${value}"`
	if (typeof value === 'number' || typeof value === 'boolean') {
		return String(value)
	}
	if (Array.isArray(value)) return `[${value.length}]`
	if (typeof value === 'object') return '{…}'
	return String(value)
}

/** `{{ connectors.<connectorId>.output }}`, or `.output.<path>` /
 * `.output[<i>]…` once a path was picked — `path` already carries its own
 * leading `[` when it starts with an array index. */
export function buildConnectorExpression(
	connectorId: string,
	path: string,
): string {
	const suffix = path === '' ? '' : path.startsWith('[') ? path : `.${path}`
	return `{{ connectors.${connectorId}.output${suffix} }}`
}
