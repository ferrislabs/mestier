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
import type { SelectOption } from '#/pages/cursus/shared/lib/options'
import { OptionSelect } from '#/pages/cursus/shared/ui/option-select'
import {
	type TriadFormValues,
	triadFormErrors,
} from '#/pages/cursus/triads/lib/triad-form'

interface TriadFormDialogProps {
	open: boolean
	fixedPromotionId: string | undefined
	promotions: SelectOption[]
	apprentices: SelectOption[]
	members: SelectOption[]
	isLoadingOptions: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: TriadFormValues) => void
}

export function TriadFormDialog({
	open,
	onOpenChange,
	...body
}: TriadFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="sm:max-w-md">
				<TriadFormBody {...body} onCancel={() => onOpenChange(false)} />
			</DialogContent>
		</Dialog>
	)
}

interface TriadFormBodyProps
	extends Omit<TriadFormDialogProps, 'open' | 'onOpenChange'> {
	onCancel: () => void
}

function TriadFormBody({
	fixedPromotionId,
	promotions,
	apprentices,
	members,
	isLoadingOptions,
	isPending,
	error,
	onCancel,
	onSubmit,
}: TriadFormBodyProps) {
	const [values, setValues] = useState<TriadFormValues>({
		promotionId: fixedPromotionId ?? '',
		apprenticeId: '',
		pedagogicTutorMemberId: null,
	})
	const [revealed, setRevealed] = useState(false)
	const errors = triadFormErrors(values)
	const shown = revealed ? errors : {}

	function submit(event: FormEvent) {
		event.preventDefault()
		setRevealed(true)
		if (Object.keys(errors).length > 0 || isPending) return
		onSubmit(values)
	}

	return (
		<form onSubmit={submit} noValidate className="grid gap-4">
			<DialogHeader>
				<DialogTitle>Nouveau trinôme</DialogTitle>
				<DialogDescription>
					Rattachez un apprenti à une promotion. Les contrats s’ajoutent ensuite
					depuis le trinôme.
				</DialogDescription>
			</DialogHeader>
			{fixedPromotionId ? null : (
				<FormField
					id="triad-promotion"
					label="Promotion"
					required
					error={shown.promotionId}
				>
					<OptionSelect
						id="triad-promotion"
						value={values.promotionId || null}
						options={promotions}
						placeholder="Choisir une promotion"
						onChange={(next) =>
							setValues((previous) => ({
								...previous,
								promotionId: next ?? '',
							}))
						}
					/>
				</FormField>
			)}
			<FormField
				id="triad-apprentice"
				label="Apprenti"
				required
				error={shown.apprenticeId}
				hint={
					isLoadingOptions
						? 'Chargement des apprentis…'
						: apprentices.length === 0
							? 'Aucun apprenti disponible. Créez-en un dans Participants.'
							: undefined
				}
			>
				<OptionSelect
					id="triad-apprentice"
					value={values.apprenticeId || null}
					options={apprentices}
					placeholder="Choisir un apprenti"
					onChange={(next) =>
						setValues((previous) => ({ ...previous, apprenticeId: next ?? '' }))
					}
				/>
			</FormField>
			<FormField id="triad-tutor" label="Tuteur pédagogique">
				<OptionSelect
					id="triad-tutor"
					value={values.pedagogicTutorMemberId}
					options={members}
					placeholder="Aucun"
					noneLabel="Aucun"
					onChange={(next) =>
						setValues((previous) => ({
							...previous,
							pedagogicTutorMemberId: next,
						}))
					}
				/>
			</FormField>
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
					Créer le trinôme
				</Button>
			</DialogFooter>
		</form>
	)
}
