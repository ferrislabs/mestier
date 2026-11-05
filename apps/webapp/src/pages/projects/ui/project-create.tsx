import { Link } from '@tanstack/react-router'
import { type FormEvent, useId } from 'react'
import { Button } from '#/components/ui/button'
import type { CustomerContext } from '#/hooks/use-customers'
import { buildOrgPath } from '#/modules/org-path'
import { hasErrors } from '#/pages/projects/lib/form-model'
import type { Person } from '#/pages/projects/lib/model'
import {
	type ProjectFullForm,
	projectFormErrors,
} from '#/pages/projects/lib/project-form'
import { useFormBinding } from '#/pages/projects/lib/use-form-binding'
import { Page, PageHeader } from '#/pages/projects/ui/page'
import {
	ProjectFields,
	type ProjectOption,
} from '#/pages/projects/ui/project-fields'

export interface ProjectCreateProps {
	organizationSlug: string
	form: ProjectFullForm
	customers: readonly ProjectOption[]
	quotes: readonly ProjectOption[]
	customerContexts: CustomerContext[]
	isCustomerContextsLoading: boolean
	people: readonly Person[]
	isPending: boolean
	error: string | null
	onChange: (patch: Partial<ProjectFullForm>) => void
	onSubmit: () => void
}

export function ProjectCreate({
	organizationSlug,
	form,
	customers,
	quotes,
	customerContexts,
	isCustomerContextsLoading,
	people,
	isPending,
	error,
	onChange,
	onSubmit,
}: ProjectCreateProps) {
	const errors = projectFormErrors(form)
	const { bind, baseId, revealAll, formRef } = useFormBinding(errors)
	const formId = useId()

	function submit(event: FormEvent) {
		event.preventDefault()
		revealAll()
		if (hasErrors(errors) || isPending) return
		onSubmit()
	}

	return (
		<Page className="max-w-3xl">
			<PageHeader
				title="Nouveau projet"
				description="Un projet regroupe des missions et peut recevoir un DOE."
			/>
			<form
				id={formId}
				ref={formRef}
				onSubmit={submit}
				noValidate
				className="rounded-lg border bg-card"
			>
				<div className="grid gap-4 p-5 sm:grid-cols-2">
					<ProjectFields
						form={form}
						onChange={onChange}
						bind={bind}
						baseId={baseId}
						customers={customers}
						quotes={quotes}
						customerContexts={customerContexts}
						isCustomerContextsLoading={isCustomerContextsLoading}
						people={people}
					/>
					{errors.other.length > 0 && (
						<ul className="grid gap-1 text-[13px] text-state-alert sm:col-span-2">
							{errors.other.map((message) => (
								<li key={message}>{message}</li>
							))}
						</ul>
					)}
					{error ? (
						<p
							role="alert"
							className="text-[13px] text-state-alert sm:col-span-2"
						>
							{error}
						</p>
					) : null}
				</div>
				<div className="flex items-center justify-between gap-3 border-t px-5 py-3">
					<p className="text-[12px] text-muted-foreground">
						Les champs marqués d’une * sont obligatoires.
					</p>
					<div className="flex gap-2">
						<Button type="button" variant="outline" asChild>
							<Link to={buildOrgPath(organizationSlug, '/projet')}>
								Annuler
							</Link>
						</Button>
						<Button type="submit" disabled={isPending}>
							Créer le projet
						</Button>
					</div>
				</div>
			</form>
		</Page>
	)
}
