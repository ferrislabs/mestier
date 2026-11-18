import { useForm } from '@tanstack/react-form'
import type { ReactNode } from 'react'
import { Button } from '#/components/ui/button'
import { Field } from '#/components/ui/field'
import { Input } from '#/components/ui/input'
import {
	type CampaignFormValues,
	campaignFormErrors,
	hasCampaignErrors,
} from '#/pages/survey/lib/campaign-form'
import { newMetadataEntry } from '#/pages/survey/lib/metadata'
import { MetadataEditor } from '#/pages/survey/ui/metadata-editor'
import { ErrorBanner } from '#/pages/survey/ui/surveys-list'

function FieldError({ message }: { message: string | undefined }) {
	return message ? <p className="text-destructive text-xs">{message}</p> : null
}

export function CampaignFormFeature({
	initialValues,
	submitLabel,
	readOnly,
	isPending,
	error,
	onSubmit,
	secondaryAction,
}: {
	initialValues: CampaignFormValues
	submitLabel: string
	readOnly: boolean
	isPending: boolean
	error: string | null
	onSubmit: (values: CampaignFormValues) => Promise<void>
	secondaryAction?: ReactNode
}) {
	const form = useForm({
		defaultValues: initialValues,
		validators: {
			onSubmit: ({ value }) => {
				const errors = campaignFormErrors(value)
				return hasCampaignErrors(errors) ? errors : undefined
			},
		},
		onSubmit: async ({ value }) => {
			await onSubmit(value)
		},
	})

	return (
		<form
			noValidate
			className="grid gap-5"
			onSubmit={(event) => {
				event.preventDefault()
				void form.handleSubmit().catch(() => undefined)
			}}
		>
			<form.Subscribe
				selector={(state) => ({
					values: state.values,
					submitted: state.submissionAttempts > 0,
				})}
			>
				{({ values, submitted }) => {
					const errors = submitted ? campaignFormErrors(values) : {}
					return (
						<>
							<form.Field name="name">
								{(field) => (
									<Field label="Nom" htmlFor="campaign-name">
										<Input
											id="campaign-name"
											value={field.state.value}
											disabled={readOnly}
											aria-invalid={Boolean(errors.name)}
											placeholder="Session de formation de mai"
											onChange={(event) =>
												field.handleChange(event.target.value)
											}
										/>
										<FieldError message={errors.name} />
									</Field>
								)}
							</form.Field>
							<div className="grid gap-4 sm:grid-cols-2">
								<form.Field name="startsAt">
									{(field) => (
										<Field label="Début" htmlFor="campaign-start">
											<Input
												id="campaign-start"
												type="datetime-local"
												value={field.state.value}
												disabled={readOnly}
												aria-invalid={Boolean(errors.startsAt)}
												onChange={(event) =>
													field.handleChange(event.target.value)
												}
											/>
											<FieldError message={errors.startsAt} />
										</Field>
									)}
								</form.Field>
								<form.Field name="endsAt">
									{(field) => (
										<Field label="Fin (facultative)" htmlFor="campaign-end">
											<Input
												id="campaign-end"
												type="datetime-local"
												value={field.state.value}
												disabled={readOnly}
												aria-invalid={Boolean(errors.endsAt)}
												onChange={(event) =>
													field.handleChange(event.target.value)
												}
											/>
											<FieldError message={errors.endsAt} />
										</Field>
									)}
								</form.Field>
							</div>
							<p className="text-muted-foreground text-xs">
								Les réponses de ses questionnaires sont acceptées à partir du
								début et refusées après la fin.
							</p>
							<form.Field name="metadata">
								{(field) => (
									<Field label="Métadonnées privées" htmlFor={null}>
										<MetadataEditor
											entries={field.state.value}
											errors={errors.metadata ?? []}
											readOnly={readOnly}
											onChange={field.handleChange}
											onAdd={() =>
												field.handleChange([
													...field.state.value,
													newMetadataEntry(),
												])
											}
										/>
									</Field>
								)}
							</form.Field>
						</>
					)
				}}
			</form.Subscribe>
			<ErrorBanner message={error} />
			<div className="flex flex-wrap justify-end gap-2">
				{secondaryAction}
				{readOnly ? null : (
					<Button type="submit" disabled={isPending}>
						{submitLabel}
					</Button>
				)}
			</div>
		</form>
	)
}
