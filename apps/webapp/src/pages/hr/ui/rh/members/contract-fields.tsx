import { useState } from 'react'
import { FormField } from '#/components/form-field'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import {
	type ContractFieldKey,
	groupContractErrors,
	isFixedTerm,
} from '#/pages/hr/lib/rh/contract-draft'
import { CONTRACT_LABELS } from '#/pages/hr/lib/rh/contracts'
import { todayIso } from '#/pages/hr/lib/rh/dates'
import type { ContractFailure } from '#/pages/hr/lib/rh/member-wire'
import { followStart } from '#/pages/hr/lib/rh/schedule-mode'
import type { ContractDraft, ContractType } from '#/pages/hr/lib/rh/types'
import { ScheduleModeEditor } from '#/pages/hr/ui/rh/members/schedule-mode-editor'

export function ContractFields({
	idPrefix,
	value,
	onChange,
	errors,
	failure = null,
	startLabel = 'Date de début',
}: {
	idPrefix: string
	value: ContractDraft
	onChange: (draft: ContractDraft) => void
	errors: readonly string[]
	failure?: ContractFailure | null
	startLabel?: string
}) {
	const [touched, setTouched] = useState<ReadonlySet<ContractFieldKey>>(
		new Set(),
	)
	const grouped = groupContractErrors(errors)
	const fixedTerm = isFixedTerm(value.type)

	function touch(...fields: ContractFieldKey[]) {
		setTouched((current) => new Set([...current, ...fields]))
	}

	function change(
		patch: Partial<ContractDraft>,
		...fields: ContractFieldKey[]
	) {
		const schedule =
			patch.startDate !== undefined
				? followStart(value.schedule, value.startDate, patch.startDate)
				: value.schedule
		onChange({ ...value, schedule, ...patch })
		if (fields.length > 0) touch(...fields)
	}

	function shown(field: ContractFieldKey): string | undefined {
		if (failure?.field === field && !failure.overlap) return failure.message
		return touched.has(field) ? grouped.byField[field] : undefined
	}
	const overlap = grouped.overlap ?? (failure?.overlap ? failure.message : null)

	function fieldProps(field: ContractFieldKey) {
		const error = shown(field)
		return {
			'aria-invalid': error ? true : undefined,
			'aria-describedby': error ? `${idPrefix}-${field}-error` : undefined,
			onBlur: () => touch(field),
		}
	}

	const id = (field: string) => `${idPrefix}-${field}`
	const today = todayIso()

	return (
		<div className="grid gap-5">
			<div className="grid gap-3 sm:grid-cols-2">
				<FormField id={id('type')} label="Type de contrat" required>
					<Select
						value={value.type}
						onValueChange={(type) =>
							change(
								{ type: type as ContractType },
								...(isFixedTerm(type as ContractType)
									? (['endDate'] as const)
									: []),
							)
						}
					>
						<SelectTrigger id={id('type')} className="w-full">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							{Object.entries(CONTRACT_LABELS).map(([type, label]) => (
								<SelectItem key={type} value={type}>
									{label}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
				</FormField>
				<FormField
					id={id('startDate')}
					label={startLabel}
					required
					error={shown('startDate')}
				>
					<Input
						id={id('startDate')}
						type="date"
						value={value.startDate}
						onChange={(event) =>
							change({ startDate: event.target.value }, 'startDate')
						}
						{...fieldProps('startDate')}
					/>
				</FormField>
				<FormField
					id={id('endDate')}
					label="Date de fin"
					required={fixedTerm}
					error={shown('endDate')}
					hint={
						fixedTerm ? undefined : 'Laisse vide si aucune fin n’est prévue.'
					}
				>
					<Input
						id={id('endDate')}
						type="date"
						value={value.endDate ?? ''}
						onChange={(event) =>
							change({ endDate: event.target.value || null }, 'endDate')
						}
						{...fieldProps('endDate')}
					/>
				</FormField>
				<FormField
					id={id('trialEndDate')}
					label="Fin de période d’essai"
					error={shown('trialEndDate')}
				>
					<Input
						id={id('trialEndDate')}
						type="date"
						value={value.trialEndDate ?? ''}
						onChange={(event) =>
							change(
								{ trialEndDate: event.target.value || null },
								'trialEndDate',
							)
						}
						{...fieldProps('trialEndDate')}
					/>
				</FormField>
				{overlap && (
					<div className="grid gap-0.5 sm:col-span-2">
						<p className="text-[13px] text-state-alert">{overlap}</p>
						<p className="text-[12px] text-muted-foreground">
							Ajuste les dates, ou renseigne d’abord la fin du contrat
							précédent.
						</p>
					</div>
				)}
			</div>
			<section aria-label="Rythme de travail" className="grid gap-3">
				<div>
					<h3 className="font-medium text-sm">Rythme de travail</h3>
					<p className="text-[12px] text-muted-foreground">
						Les jours et horaires de ce contrat, repris dans le planning.
					</p>
				</div>
				<ScheduleModeEditor
					idPrefix={id('schedule')}
					value={value.schedule}
					onChange={(schedule) => change({ schedule })}
					contractStart={value.startDate || today}
					today={today}
				/>
			</section>
		</div>
	)
}
