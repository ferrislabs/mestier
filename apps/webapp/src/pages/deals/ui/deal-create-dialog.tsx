import { Link } from '@tanstack/react-router'
import { type ReactNode, useId, useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from '#/components/ui/dialog'
import type { Customer } from '#/hooks/use-customers'
import { buildOrgPath } from '#/modules/org-path'
import {
	buildDraft,
	type DealForm,
	emptyForm,
} from '#/pages/deals/lib/deal-form'
import type { DealDraft } from '#/pages/deals/lib/deal-types'
import { accountOptions } from '#/pages/deals/lib/pipeline-filters'
import { DealFields } from '#/pages/deals/ui/deal-fields'

export function DealCreateDialog({
	open,
	onOpenChange,
	trigger,
	customers,
	defaultOwnerId,
	organizationSlug,
	onCreate,
}: {
	open: boolean
	onOpenChange: (open: boolean) => void
	trigger?: ReactNode
	customers: readonly Customer[]
	defaultOwnerId: string
	organizationSlug: string
	onCreate: (draft: DealDraft) => void
}) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			{trigger && <DialogTrigger asChild>{trigger}</DialogTrigger>}
			<DialogContent className="flex max-h-[calc(100dvh-2rem)] flex-col gap-0 p-0 sm:max-w-xl">
				<DealCreateForm
					customers={customers}
					defaultOwnerId={defaultOwnerId}
					organizationSlug={organizationSlug}
					onCancel={() => onOpenChange(false)}
					onCreate={(draft) => {
						onCreate(draft)
						onOpenChange(false)
					}}
				/>
			</DialogContent>
		</Dialog>
	)
}

function DealCreateForm({
	customers,
	defaultOwnerId,
	organizationSlug,
	onCancel,
	onCreate,
}: {
	customers: readonly Customer[]
	defaultOwnerId: string
	organizationSlug: string
	onCancel: () => void
	onCreate: (draft: DealDraft) => void
}) {
	const idPrefix = useId()
	const [form, setForm] = useState<DealForm>(() => emptyForm(defaultOwnerId))
	const [submitted, setSubmitted] = useState(false)
	const result = buildDraft(form)

	return (
		<form
			noValidate
			className="flex min-h-0 flex-1 flex-col"
			onSubmit={(event) => {
				event.preventDefault()
				setSubmitted(true)
				if (result.draft) onCreate(result.draft)
			}}
		>
			<DialogHeader className="px-6 pt-6 pb-4">
				<DialogTitle>Nouvelle opportunité</DialogTitle>
				<DialogDescription>
					Elle arrive dans la colonne Nouvelle demande, tu pourras la faire
					avancer ensuite.
				</DialogDescription>
			</DialogHeader>
			<div className="min-h-0 flex-1 overflow-y-auto border-t px-6 py-5">
				<DealFields
					idPrefix={idPrefix}
					form={form}
					errors={submitted ? result.errors : {}}
					customers={accountOptions(customers, null)}
					onChange={(patch) => setForm((current) => ({ ...current, ...patch }))}
					clientNote={
						<>
							Le client n’existe pas encore ?{' '}
							<Link
								to={buildOrgPath(organizationSlug, '/crm/clients')}
								className="rounded-sm text-primary underline-offset-2 outline-none hover:underline focus-visible:ring-2 focus-visible:ring-ring"
							>
								Crée-le dans Clients
							</Link>
						</>
					}
				/>
			</div>
			<DialogFooter className="border-t px-6 py-4">
				<Button type="button" variant="outline" onClick={onCancel}>
					Annuler
				</Button>
				<Button type="submit">Créer l’opportunité</Button>
			</DialogFooter>
		</form>
	)
}
