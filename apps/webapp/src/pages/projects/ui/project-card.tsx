import { type FormEvent, useId, useState } from 'react'
import { ConfirmDialog } from '#/components/confirm-dialog'
import { FormField } from '#/components/form-field'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import type { CustomerContext } from '#/hooks/use-customers'
import type { Person, ProjectStatus } from '#/pages/projects/lib/model'
import {
	canSaveFullForm,
	customerIdOfSelect,
	customerSelectValue,
	datesError,
	NO_QUOTE,
	type ProjectFullForm,
	quoteIdOfSelect,
	quoteSelectValue,
	sameFullForm,
	validateProjectFields,
} from '#/pages/projects/lib/project-form'
import { CLIENT_INTERNAL } from '#/pages/projects/lib/project-list'
import { saveStatus } from '#/pages/projects/lib/save-status'
import { ProjectStatusBadge } from '#/pages/projects/ui/badges'
import { PersonSelect } from '#/pages/projects/ui/person-select'
import type { ProjectOption } from '#/pages/projects/ui/project-fields'
import { ProjectTextarea } from '#/pages/projects/ui/project-textarea'
import { SettingsSection } from '#/pages/projects/ui/settings-section'
import { useReportDirty } from '#/pages/projects/ui/unsaved-guard'
import { BillingAddressField } from '#/pages/quotes/ui/billing-address-field'

export interface ProjectCardProps {
	projectName: string
	status: ProjectStatus
	form: ProjectFullForm
	stored: ProjectFullForm
	customers: readonly ProjectOption[]
	quotes: readonly ProjectOption[]
	customerContexts: CustomerContext[]
	isCustomerContextsLoading: boolean
	people: readonly Person[]
	isSaving: boolean
	error: string | null
	onChange: (patch: Partial<ProjectFullForm>) => void
	onSubmit: () => Promise<void>
	onToggleArchive: () => void
	onDirtyChange: (dirty: boolean) => void
}

export function ProjectCard({
	projectName,
	status,
	form,
	stored,
	customers,
	quotes,
	customerContexts,
	isCustomerContextsLoading,
	people,
	isSaving,
	error,
	onChange,
	onSubmit,
	onToggleArchive,
	onDirtyChange,
}: ProjectCardProps) {
	const baseId = useId()
	const [saved, setSaved] = useState(false)
	const [archiving, setArchiving] = useState(false)
	const dirty = !sameFullForm(form, stored)
	const archived = status === 'archived'
	const nameError = validateProjectFields({
		...form,
		startDate: '',
		endDate: '',
	})[0]
	const dates = datesError(form)
	const hasCustomer = form.customerId !== ''
	useReportDirty(dirty, onDirtyChange)

	function change(patch: Partial<ProjectFullForm>) {
		onChange(patch)
		setSaved(false)
	}

	function submit(event: FormEvent) {
		event.preventDefault()
		if (!canSaveFullForm(form, stored) || isSaving) return
		void onSubmit().then(() => setSaved(true))
	}

	const sorted = [...customers].sort((a, b) =>
		a.label.localeCompare(b.label, 'fr'),
	)

	return (
		<SettingsSection
			id={`${baseId}-project`}
			title="Projet"
			description="Les informations du projet, reprises dans l’aperçu et la liste des projets."
			actions={<ProjectStatusBadge status={status} />}
		>
			<form onSubmit={submit} noValidate>
				<div className="grid gap-4 px-5 py-5 sm:grid-cols-2">
					<FormField
						id={`${baseId}-name`}
						label="Nom"
						required
						error={nameError}
					>
						<Input
							id={`${baseId}-name`}
							value={form.name}
							onChange={(event) => change({ name: event.target.value })}
							autoComplete="off"
							aria-invalid={nameError ? true : undefined}
							aria-describedby={nameError ? `${baseId}-name-error` : undefined}
						/>
					</FormField>
					<FormField id={`${baseId}-customer`} label="Client">
						<Select
							value={customerSelectValue(form.customerId)}
							onValueChange={(value) =>
								change({
									customerId: customerIdOfSelect(value),
									quoteId: '',
									customerContextId: '',
								})
							}
						>
							<SelectTrigger id={`${baseId}-customer`} className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value={CLIENT_INTERNAL}>
									Projet interne, sans client
								</SelectItem>
								{sorted.map((customer) => (
									<SelectItem key={customer.id} value={customer.id}>
										{customer.label}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</FormField>
					<FormField
						id={`${baseId}-address`}
						label="Adresse de facturation"
						hint="Requise pour émettre un acompte ou un solde depuis ce projet."
					>
						<BillingAddressField
							id={`${baseId}-address`}
							value={form.customerContextId}
							addresses={customerContexts}
							hasCustomer={hasCustomer}
							isLoading={isCustomerContextsLoading}
							onChange={(customerContextId) => change({ customerContextId })}
						/>
					</FormField>
					<FormField
						id={`${baseId}-quote`}
						label="Devis"
						hint={
							hasCustomer
								? 'La marge est le devis moins le coût planifié.'
								: 'Un devis suppose un client.'
						}
					>
						<Select
							value={quoteSelectValue(form.quoteId)}
							onValueChange={(value) =>
								change({ quoteId: quoteIdOfSelect(value) })
							}
							disabled={!hasCustomer}
						>
							<SelectTrigger id={`${baseId}-quote`} className="w-full">
								<SelectValue />
							</SelectTrigger>
							<SelectContent>
								<SelectItem value={NO_QUOTE}>
									Aucun, pas de marge calculée
								</SelectItem>
								{quotes.map((quote) => (
									<SelectItem key={quote.id} value={quote.id}>
										{quote.label}
									</SelectItem>
								))}
							</SelectContent>
						</Select>
					</FormField>
					<FormField
						id={`${baseId}-place`}
						label="Lieu d’intervention"
						className="sm:col-span-2"
					>
						<Input
							id={`${baseId}-place`}
							value={form.place}
							onChange={(event) => change({ place: event.target.value })}
							autoComplete="off"
						/>
					</FormField>
					<FormField
						id={`${baseId}-description`}
						label="Description"
						className="sm:col-span-2"
					>
						<ProjectTextarea
							id={`${baseId}-description`}
							rows={3}
							value={form.description}
							onChange={(event) => change({ description: event.target.value })}
						/>
					</FormField>
					<FormField id={`${baseId}-start`} label="Début">
						<Input
							id={`${baseId}-start`}
							type="date"
							value={form.startDate}
							onChange={(event) => change({ startDate: event.target.value })}
							aria-invalid={dates ? true : undefined}
							aria-describedby={dates ? `${baseId}-dates` : undefined}
						/>
					</FormField>
					<FormField id={`${baseId}-end`} label="Fin">
						<Input
							id={`${baseId}-end`}
							type="date"
							value={form.endDate}
							onChange={(event) => change({ endDate: event.target.value })}
							aria-invalid={dates ? true : undefined}
							aria-describedby={dates ? `${baseId}-dates` : undefined}
						/>
					</FormField>
					{dates && (
						<p
							id={`${baseId}-dates`}
							className="text-[12px] text-state-alert sm:col-span-2"
						>
							{dates}
						</p>
					)}
					<FormField id={`${baseId}-owner`} label="Responsable">
						<PersonSelect
							id={`${baseId}-owner`}
							value={form.ownerId}
							people={people}
							onChange={(ownerId) => change({ ownerId })}
						/>
					</FormField>
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
					<output className="text-[13px] text-muted-foreground">
						{saveStatus({
							dirty,
							saved,
							savedLabel: 'Modifications enregistrées.',
						})}
					</output>
					<Button
						type="submit"
						size="sm"
						disabled={!canSaveFullForm(form, stored) || isSaving}
					>
						Enregistrer
					</Button>
				</div>
			</form>
			<div className="flex flex-wrap items-center justify-between gap-3 border-t px-5 py-4">
				<div className="min-w-0">
					<h3 className="font-medium text-sm">
						{archived ? 'Projet archivé' : 'Archiver le projet'}
					</h3>
					<p className="mt-0.5 text-[13px] text-muted-foreground">
						{archived
							? 'Le projet est sorti de la liste des projets actifs. Réactive-le pour reprendre le travail.'
							: 'Le projet quitte la liste des projets actifs. Ses missions et son DOE sont conservés.'}
					</p>
				</div>
				<Button
					type="button"
					variant="outline"
					size="sm"
					onClick={() => setArchiving(true)}
				>
					{archived ? 'Réactiver' : 'Archiver le projet'}
				</Button>
			</div>
			<ConfirmDialog
				open={archiving}
				onOpenChange={setArchiving}
				title={archived ? 'Réactiver le projet ?' : 'Archiver le projet ?'}
				description={
					archived
						? `« ${projectName} » revient dans la liste des projets actifs.`
						: `« ${projectName} » quitte la liste des projets actifs. Tu pourras le réactiver à tout moment.`
				}
				confirmLabel={archived ? 'Réactiver' : 'Archiver'}
				onConfirm={() => {
					onToggleArchive()
					setArchiving(false)
				}}
			/>
		</SettingsSection>
	)
}
