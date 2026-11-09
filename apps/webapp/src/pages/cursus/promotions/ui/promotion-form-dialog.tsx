import { type FormEvent, useState } from 'react'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Input } from '#/components/ui/input'
import {
	type PromotionFormValues,
	promotionFormErrors,
} from '#/pages/cursus/promotions/lib/promotion-form'
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'

interface PromotionFormDialogProps {
	open: boolean
	title: string
	submitLabel: string
	initial: PromotionFormValues
	trainings: SelectOption[]
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: PromotionFormValues) => void
	onManageTrainings?: () => void
}

export function PromotionFormDialog({
	open,
	onOpenChange,
	...body
}: PromotionFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				<PromotionFormBody {...body} onCancel={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

interface PromotionFormBodyProps
	extends Omit<PromotionFormDialogProps, 'open' | 'onOpenChange'> {
	onCancel: () => void
}

function PromotionFormBody({
	title,
	submitLabel,
	initial,
	trainings,
	isPending,
	error,
	onCancel,
	onSubmit,
	onManageTrainings,
}: PromotionFormBodyProps) {
	const [values, setValues] = useState(initial)
	const [revealed, setRevealed] = useState(false)
	const errors = promotionFormErrors(values)
	const shown = revealed ? errors : {}

	function patch(next: Partial<PromotionFormValues>) {
		setValues((previous) => ({ ...previous, ...next }))
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		setRevealed(true)
		if (Object.keys(errors).length > 0 || isPending) return
		onSubmit(values)
	}

	return (
		<form onSubmit={submit} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>{title}</DialogTitle>
				<DialogDescription>
					Une promotion regroupe les apprentis d’une formation pour une période.
				</DialogDescription>
			</DialogHeader>
			<FormField
				id="promotion-training"
				label="Formation"
				required
				error={shown.trainingId}
			>
				<OptionSelect
					id="promotion-training"
					value={values.trainingId || null}
					options={trainings}
					placeholder="Choisir une formation"
					onChange={(next) => patch({ trainingId: next ?? '' })}
				/>
				{onManageTrainings ? (
					<p className="text-muted-foreground text-xs">
						{trainings.length === 0
							? 'Aucune formation pour l’instant. '
							: 'Une formation manque ? '}
						<button
							type="button"
							className="text-foreground underline underline-offset-2"
							onClick={onManageTrainings}
						>
							Gérer les formations dans Paramètres › Cursus
						</button>
					</p>
				) : null}
			</FormField>
			<FormField
				id="promotion-label"
				label="Libellé"
				required
				error={shown.label}
			>
				<Input
					id="promotion-label"
					value={values.label}
					onChange={(event) => patch({ label: event.target.value })}
				/>
			</FormField>
			<div className="grid grid-cols-2 gap-3">
				<FormField
					id="promotion-start"
					label="Année de début"
					required
					error={shown.startYear}
				>
					<Input
						id="promotion-start"
						inputMode="numeric"
						value={values.startYear}
						onChange={(event) => patch({ startYear: event.target.value })}
					/>
				</FormField>
				<FormField
					id="promotion-end"
					label="Année de fin"
					required
					error={shown.endYear}
				>
					<Input
						id="promotion-end"
						inputMode="numeric"
						value={values.endYear}
						onChange={(event) => patch({ endYear: event.target.value })}
					/>
				</FormField>
			</div>
			{error ? (
				<p role="alert" className="text-[13px] text-state-alert">
					{error}
				</p>
			) : null}
			<DialogFooter>
				<Button type="button" variant="outline" onClick={onCancel}>
					Annuler
				</Button>
				<Button type="submit" disabled={isPending}>
					{submitLabel}
				</Button>
			</DialogFooter>
		</form>
	)
}
