import type { Schemas } from '#/api/api.client'

export type DafStatus = Schemas.DafStatus

export const STATUS_OPTIONS: ReadonlyArray<{
	value: DafStatus
	label: string
}> = [
	{ value: 'pending', label: 'En attente' },
	{ value: 'accepted', label: 'Acceptée' },
	{ value: 'rejected_with_comments', label: 'Refus avec obs.' },
	{ value: 'rejected', label: 'Refus ferme' },
]

const CLASS_NAMES: Record<DafStatus, string> = {
	pending: 'bg-amber-100 text-amber-800',
	accepted: 'bg-success-soft text-success',
	rejected_with_comments: 'bg-orange-100 text-orange-800',
	rejected: 'bg-destructive-soft text-destructive',
}

export function statusLabel(status: DafStatus): string {
	return STATUS_OPTIONS.find((option) => option.value === status)?.label ?? ''
}

export function statusClassName(status: DafStatus): string {
	return CLASS_NAMES[status]
}

export function dafNumberLabel(number: number): string {
	return `D.A.F n°${String(number).padStart(2, '0')}`
}

export function formatDate(value: string | null | undefined): string {
	if (!value) return '–'
	const [year, month, day] = value.slice(0, 10).split('-')
	return year && month && day ? `${day}/${month}/${year}` : value
}
