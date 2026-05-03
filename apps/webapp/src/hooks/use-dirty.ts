import { useCallback, useMemo, useState } from 'react'

export interface DirtyResult<T> {
	isDirty: boolean
	changedKeys: (keyof T)[]
}

export function useDirty<T extends object>(
	current: T,
	initial: T,
): DirtyResult<T> {
	return useMemo(() => diff(current, initial), [current, initial])
}

export interface DirtyBaseline<T extends object> {
	baseline: T
	isDirty: boolean
	changedKeys: (keyof T)[]
	commit: (value?: T) => void
	reset: () => void
}

export function useDirtyBaseline<T extends object>(
	initial: T,
	current: T,
): DirtyBaseline<T> {
	const [original, setOriginal] = useState<T>(initial)
	const [baseline, setBaseline] = useState<T>(initial)

	if (original !== initial) {
		setOriginal(initial)
		setBaseline(initial)
	}

	const { isDirty, changedKeys } = useMemo(
		() => diff(current, baseline),
		[current, baseline],
	)

	const commit = useCallback(
		(value?: T) => {
			setBaseline(value ?? current)
		},
		[current],
	)

	const reset = useCallback(() => {
		setBaseline(original)
	}, [original])

	return { baseline, isDirty, changedKeys, commit, reset }
}

function diff<T extends object>(current: T, initial: T): DirtyResult<T> {
	const changedKeys: (keyof T)[] = []
	const keys = new Set<keyof T>([
		...(Object.keys(current) as (keyof T)[]),
		...(Object.keys(initial) as (keyof T)[]),
	])
	for (const key of keys) {
		if (!isEqual(current[key], initial[key])) changedKeys.push(key)
	}
	return { isDirty: changedKeys.length > 0, changedKeys }
}

function isEqual(a: unknown, b: unknown): boolean {
	if (Object.is(a, b)) return true
	if (a === null || b === null) return false
	if (typeof a !== 'object' || typeof b !== 'object') return false

	if (a instanceof Date && b instanceof Date) return a.getTime() === b.getTime()
	if (a instanceof Date || b instanceof Date) return false

	if (Array.isArray(a) || Array.isArray(b)) {
		if (!Array.isArray(a) || !Array.isArray(b)) return false
		if (a.length !== b.length) return false
		for (let i = 0; i < a.length; i++) {
			if (!isEqual(a[i], b[i])) return false
		}
		return true
	}

	const aKeys = Object.keys(a as object)
	const bKeys = Object.keys(b as object)
	if (aKeys.length !== bKeys.length) return false
	for (const k of aKeys) {
		if (!Object.hasOwn(b as object, k)) return false
		if (
			!isEqual(
				(a as Record<string, unknown>)[k],
				(b as Record<string, unknown>)[k],
			)
		) {
			return false
		}
	}
	return true
}
