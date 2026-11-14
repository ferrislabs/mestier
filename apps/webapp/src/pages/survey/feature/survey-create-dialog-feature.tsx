import { useForm } from '@tanstack/react-form'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '#/components/ui/dialog'
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import type { Visibility } from '#/pages/survey/types'
import { ErrorBanner } from '#/pages/survey/ui/surveys-list'
import { VisibilityPicker } from '#/pages/survey/ui/visibility-picker'

export interface SurveyCreateValues {
	title: string
	visibility: Visibility
}

export function SurveyCreateDialogFeature({
	open,
	isPending,
	error,
	onOpenChange,
	onSubmit,
}: {
	open: boolean
	isPending: boolean
	error: string | null
	onOpenChange: (open: boolean) => void
	onSubmit: (values: SurveyCreateValues) => Promise<void>
}) {
	const form = useForm({
		defaultValues: {
			title: '',
			visibility: 'PUBLIC',
		} as SurveyCreateValues,
		onSubmit: async ({ value }) => {
			await onSubmit({
				title: value.title.trim(),
				visibility: value.visibility,
			})
			form.reset()
		},
	})

	return (
		<Dialog
			open={open}
			onOpenChange={(next) => {
				if (!next) form.reset()
				onOpenChange(next)
			}}
		>
			<DialogContent>
				<form
					noValidate
					className="grid gap-5"
					onSubmit={(event) => {
						event.preventDefault()
						void form.handleSubmit().catch(() => undefined)
					}}
				>
					<DialogHeader>
						<DialogTitle>Nouveau questionnaire</DialogTitle>
						<DialogDescription>
							Il est créé en brouillon. Vous construirez ses questions dans
							l’éditeur.
						</DialogDescription>
					</DialogHeader>
					<form.Field
						name="title"
						validators={{
							onSubmit: ({ value }) =>
								value.trim() === ''
									? 'Donnez un titre au questionnaire.'
									: undefined,
						}}
					>
						{(field) => (
							<Field label="Titre" htmlFor="survey-title">
								<Input
									id="survey-title"
									value={field.state.value}
									onChange={(event) => field.handleChange(event.target.value)}
									onBlur={field.handleBlur}
									aria-invalid={field.state.meta.errors.length > 0}
									placeholder="Bilan de fin de session"
									autoFocus
								/>
								{field.state.meta.errors.map((message) => (
									<p key={String(message)} className="text-destructive text-xs">
										{String(message)}
									</p>
								))}
							</Field>
						)}
					</form.Field>
					<form.Field name="visibility">
						{(field) => (
							<Field label="Visibilité" htmlFor={null}>
								<VisibilityPicker
									name="create-visibility"
									value={field.state.value}
									onChange={field.handleChange}
								/>
								<p className="text-muted-foreground text-xs">
									La visibilité ne change plus une fois le questionnaire ouvert.
								</p>
							</Field>
						)}
					</form.Field>
					<ErrorBanner message={error} />
					<DialogFooter>
						<Button
							type="button"
							variant="outline"
							onClick={() => onOpenChange(false)}
						>
							Annuler
						</Button>
						<Button type="submit" disabled={isPending}>
							Créer le questionnaire
						</Button>
					</DialogFooter>
				</form>
			</DialogContent>
		</Dialog>
	)
}
